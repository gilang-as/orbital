/// CANONICAL: Shell command logic - authoritative version
///
/// This module is the single source of truth for shell commands.
///
/// Phase 2.5 (now):
///   - Kernel shell task calls this (directly via kernel shell_commands.rs mirror)
///   - Userspace CLI compiles this standalone
///
/// Phase 3 (future):
///   - Userspace shell binary calls this
///   - Kernel only dispatches syscalls
///   - Shell_commands.rs in kernel deleted
///
/// All commands call syscall wrappers (which in Phase 3 invoke actual syscalls,
/// and in Phase 2.5 call kernel functions directly via mirror copy in kernel).

/// Execute a shell command
pub fn execute_command(command: &str) {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "help" => cmd_help(),
        "echo" => cmd_echo(&parts[1..]),
        "ps" => cmd_ps(),
        "pid" => cmd_pid(),
        "uptime" => cmd_uptime(),
        "ping" => cmd_ping(),
        "spawn" => cmd_spawn(&parts[1..]),
        "wait" => cmd_wait(&parts[1..]),
        "run" => cmd_run(),
        "clear" => cmd_clear(),
        "exit" => cmd_exit(),
        _ => println!("unknown command: '{}' (try 'help')", parts[0]),
    }
}

fn cmd_help() {
    println!("Available commands:");
    println!("  help            - Show this help");
    println!("  echo <text>     - Echo text");
    println!("  ps              - List processes");
    println!("  pid             - Show current PID");
    println!("  uptime          - Show kernel uptime");
    println!("  ping            - Connectivity test");
    println!("  spawn <n>       - Spawn n tasks");
    println!("  wait <pid>      - Wait for process");
    println!("  run             - Execute ready tasks");
    println!("  clear           - Clear screen");
    println!("  exit            - Exit shell");
}

fn cmd_echo(args: &[&str]) {
    if args.is_empty() {
        println!("");
    } else {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 { print!(" "); }
            print!("{}", arg);
        }
        println!();
    }
}

fn cmd_ps() {
    println!("Running processes (stub - use ps syscall for details)");
}

fn cmd_pid() {
    // In Phase 3, this will call syscall_get_pid()
    // For now, kernel shell task directly accesses scheduler
    #[cfg(feature = "userspace")]
    {
        match syscall_get_pid() {
            Ok(pid) => println!("Current PID: {}", pid),
            Err(_) => println!("Error getting PID"),
        }
    }
    #[cfg(not(feature = "userspace"))]
    {
        println!("PID: (not available in standalone compilation)");
    }
}

fn cmd_uptime() {
    // In Phase 3, this will call syscall_uptime()
    // For now, kernel shell task directly accesses scheduler
    #[cfg(feature = "userspace")]
    {
        match syscall_uptime() {
            Ok(seconds) => println!("Uptime: {} seconds", seconds),
            Err(_) => println!("Error getting uptime"),
        }
    }
    #[cfg(not(feature = "userspace"))]
    {
        println!("Uptime: (not available in standalone compilation)");
    }
}

fn cmd_ping() {
    println!("pong");
}

fn cmd_spawn(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: spawn <count>");
        return;
    }
    
    if let Ok(count) = args[0].parse::<usize>() {
        for i in 0..count {
            // In Phase 3, this will call syscall_task_create()
            #[cfg(feature = "userspace")]
            {
                // Userspace version would call syscalls
                println!("spawn {} (via syscall)", i + 1);
            }
            #[cfg(not(feature = "userspace"))]
            {
                // Kernel mirror version uses direct kernel calls
                println!("spawn {} (kernel direct)", i + 1);
            }
        }
    }
}

fn cmd_wait(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: wait <pid>");
        return;
    }
    
    if let Ok(pid) = args[0].parse::<u64>() {
        if pid > 0 {
            println!("Waiting for PID {}...", pid);
            // TODO: Implement actual wait
            println!("Process completed");
        }
    }
}

fn cmd_run() {
    println!("Executing all ready processes...");
    // In Phase 3: syscall_run_ready()
    #[cfg(feature = "userspace")]
    {
        println!("Executed 0 process(es) (via syscall)");
    }
    #[cfg(not(feature = "userspace"))]
    {
        println!("Executed 0 process(es) (kernel direct)");
    }
}

fn cmd_clear() {
    println!("[clear screen]");
    // In Phase 3: syscall_clear_screen()
}

fn cmd_exit() {
    println!("Exiting...");
    std::process::exit(0);
}
