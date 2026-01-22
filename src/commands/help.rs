use anyhow::Result;

pub fn run() -> Result<()> {
    println!("proj - Project tracking and context management for AI-assisted development");
    println!();
    println!("USAGE:");
    println!("    proj <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("  init          Initialize new project");
    println!("  migrate       Migrate existing project to proj format");
    println!("  status        Show current project status");
    println!("  resume        Detailed context for resuming work");
    println!("  session       Session management (start/end/list)");
    println!("  log           Log decisions/notes/blockers/questions");
    println!("  task          Task management (add/update/list)");
    println!("  tasks         Shortcut for 'task list'");
    println!("  context       Search decisions and notes");
    println!("  delta         Show changes since last status");
    println!("  compress      Compress old sessions");
    println!("  cleanup       Clean up stale items");
    println!("  upgrade       Upgrade database schema");
    println!("  register      Register project in global registry");
    println!("  registered    List registered projects");
    println!("  dashboard     Overview of all projects");
    println!("  snapshot      Generate AI context snapshot");
    println!("  export        Export session history");
    println!("  backup        Manual backup");
    println!("  check         Verify database integrity");
    println!("  extend        Add extension tables");
    println!("  archive       Archive completed project");
    println!("  help          Show this help message");
    println!();
    println!("Run 'proj <COMMAND> --help' for more information on a command.");
    Ok(())
}
