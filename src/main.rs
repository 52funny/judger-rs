use core::panic;
use std::{
    mem::MaybeUninit,
    os::unix::process::CommandExt,
    process::{Command, Stdio},
};
use tracing::error;

fn main() {
    let timeout = 1000;
    let stack_limit = 1024 * 1024 + 2;
    let input_path: &'static str = "./1.in";
    let output_path: &'static str = "./1.out";
    tracing_subscriber::fmt::init();
    let start = std::time::Instant::now();
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        panic!("fork failed");
    }
    match pid {
        // child process
        0 => {
            #[cfg(target_os = "linux")]
            judger_rs::general_seccomp_rules();
            // set limit rules
            let limits_rules = &[
                (libc::RLIMIT_CPU, "RLIMIT_CPU", (timeout + 1000 - 1) / 1000),
                (libc::RLIMIT_STACK, "RLIMIT_STACK", stack_limit),
                #[cfg(target_os = "linux")]
                (libc::RLIMIT_AS, "RLIMIT_AS", stack_limit),
            ];
            for limit in limits_rules {
                let code = unsafe {
                    libc::setrlimit(
                        limit.0,
                        &libc::rlimit {
                            rlim_cur: limit.2,
                            rlim_max: limit.2,
                        },
                    )
                };
                if code == -1 {
                    error!(
                        "setrlimit {} failed, {}",
                        limit.1,
                        std::io::Error::last_os_error()
                    );
                }
            }
            // using exec to execute
            let stdin = match std::fs::File::open(input_path) {
                Ok(f) => f,
                Err(_) => unsafe {
                    error!("open input file error!");
                    // send SIGUSR1 signal
                    libc::raise(libc::SIGUSR1);
                    libc::exit(-1);
                },
            };
            let stdout = match std::fs::File::create(output_path) {
                Ok(f) => f,
                Err(_) => unsafe {
                    error!("create ouput file error!");
                    libc::raise(libc::SIGUSR1);
                    libc::exit(-1);
                },
            };

            Command::new("./main")
                .stdin(stdin)
                .stdout(Stdio::from(stdout.try_clone().unwrap()))
                .stderr(Stdio::from(stdout))
                .exec();
        }
        pid => {
            // kill process if it is running for more than `timeout`
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(timeout));
                unsafe {
                    libc::kill(pid, libc::SIGKILL);
                }
            });
            let mut status = -1;
            let mut rusage = MaybeUninit::<libc::rusage>::zeroed();
            if unsafe {
                libc::wait4(
                    pid,
                    &mut status as *mut i32,
                    libc::WSTOPPED,
                    rusage.as_mut_ptr(),
                )
            } == -1
            {
                unsafe { libc::kill(pid, libc::SIGKILL) };
                error!("pid: {}, wait4 error.", pid);
            };
            let end = std::time::Instant::now();

            let rusage = unsafe { rusage.assume_init() };
            let signal = if libc::WIFSIGNALED(status) {
                libc::WTERMSIG(status)
            } else {
                0
            };
            let cpu_time = rusage.ru_utime.tv_sec * 1000 + rusage.ru_utime.tv_usec as i64 / 1000;
            let exit_code = libc::WEXITSTATUS(status);
            let res = Result {
                signal,
                exit_code,
                cpu_time,
                real_time: (end - start).as_millis() as i64,
                #[cfg(target_os = "linux")]
                memory: rusage.ru_maxrss * 1024,
            };
            serde_json::to_writer_pretty(std::io::stdout(), &res).unwrap();
        }
    }
}
#[derive(Debug, serde::Serialize)]
pub struct Result {
    pub signal: i32,
    pub exit_code: i32,
    pub cpu_time: i64,
    pub real_time: i64,
    #[cfg(target_os = "linux")]
    pub memory: i64,
}
