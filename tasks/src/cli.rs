use std::process;
use std::str::FromStr;

use anyhow::Result;
use clap_complete::Shell;
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};

use crate::ctx::TaskContext;

/// Attempts to find the type of shell that invoked `aatask`
pub fn get_current_shell() -> Result<Option<Shell>> {
    let mut system = System::default();
    system.refresh_processes();

    let mut pid = Some(Pid::from_u32(process::id()));
    while pid.is_some() {
        let process = match system.process(pid.unwrap()) {
            Some(p) => p,
            None => return Ok(None),
        };
        pid = process.parent();

        // If it's not a shell, skip it
        let exe_name = process.exe().file_name().unwrap().to_str().unwrap();
        let Ok(sh) = Shell::from_str(exe_name) else {
            continue;
        };

        return Ok(Some(sh));
    }

    Ok(None)
}

pub fn generate_completion_script(
    shell: Shell,
    task_ctx: &mut TaskContext,
) -> Result<String> {
    let cmd = &mut task_ctx.command;
    let mut script = vec![];
    clap_complete::generate(shell, cmd, "aatask", &mut script);

    let script_string = match shell {
        Shell::Zsh =>
        // Rewrite the completion script so that it only calls the completion function if
        // in the context of a completion action
        {
            String::from_utf8(script)?.replace(
                "_aatask \"$@\"\n",
                "
if [ \"$funcstack[1]\" = \"_aatask\" ]; then
    _aatask \"$@\"
fi"
                .trim(),
            )
        }
        _ => String::from_utf8(script)?,
    };

    Ok(script_string)
}
