use seccomp::{Action, Compare, Rule};
use std::error::Error;

pub fn general_seccomp_rules() -> Result<(), Box<dyn Error>> {
    // references: https://filippo.io/linux-syscall-table/
    let basic_ban_rules = [
        // clone
        Rule::new(56, None, Action::Errno(libc::EPERM)),
        // fork
        Rule::new(57, None, Action::Errno(libc::EPERM)),
        // vfork
        Rule::new(58, None, Action::Errno(libc::EPERM)),
        // kill
        Rule::new(62, None, Action::Errno(libc::EPERM)),
    ];
    let mut ctx = seccomp::Context::default(Action::Allow)?;
    for r in basic_ban_rules {
        ctx.add_rule(r)?;
    }
    ctx.add_rule(Rule::new(
        105,
        Compare::arg(0).with(1000).using(seccomp::Op::Eq).build(),
        Action::Errno(libc::EPERM),
    ))?;
    ctx.load()?;
    Ok(())
}
