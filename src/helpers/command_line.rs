use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    ExecutableCommand,
};
use std::io::{stdin, stdout};

#[derive(PartialEq, Debug)]
pub enum PrintCommand {
    AICall,
    UnitTest,
    Issue,
}

impl PrintCommand {
    pub fn print_agent_message(&self, agent_pos: &str, agent_statement: &str) {
        let mut stdout: std::io::Stdout = stdout();

        // decide on the print color
        let statement_color: Color = match self {
            Self::AICall => Color::Cyan,
            Self::UnitTest => Color::Magenta,
            Self::Issue => Color::Red,
        };

        // print agent statement to stdou
        stdout.execute(SetForegroundColor(Color::Green)).unwrap();
        print!("Agent: {:} ", agent_pos);
        stdout.execute(SetForegroundColor(statement_color)).unwrap();
        println!("{}", agent_statement);

        // reset color
        stdout.execute(ResetColor).unwrap();
    }
}
// get user response that code is safe to execute
pub fn confirm_safe_code() -> bool {
    
    let mut stdout: std::io::Stdout = stdout();
    loop {
        stdout.execute( SetForegroundColor(Color::DarkYellow) ).unwrap();
        println!("WARNING: You are about to run code written entirely by AI.");
        println!("Review your code and confirm that you wish to continue.");

        stdout.execute(ResetColor).unwrap();

        stdout.execute( SetForegroundColor(Color::Green) ).unwrap();

        println!("[1] All Good");

        stdout.execute( SetForegroundColor(Color::Red) ).unwrap();

        println!("[2] Lets stop this project");

        stdout.execute(ResetColor);

        let mut human_response : String = String::new();

        stdin()
            .read_line( &mut human_response )
            .expect( "Failed to read response");

        let human_response = human_response.trim().to_lowercase();

        match human_response.as_str() {
            
            "1" | "ok" | "y" | "yes" => return true,
            "2" | "no" | "n" => return false,
            _ => {
                println!("Invalid input, please select [1] or [2]");
            }
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests_prints_agent_message() {
        PrintCommand::AICall
            .print_agent_message("Managing Agent", "Testing testing, processing something");
    }
}

pub fn get_user_response(question: &str) -> String {
    let mut stdout: std::io::Stdout = stdout();

    stdout.execute(SetForegroundColor(Color::Blue)).unwrap();
    println!("");
    println!("{}", question);

    // reset the color
    stdout.execute(ResetColor).unwrap();

    // read the user response
    let mut user_response: String = String::new();
    stdin()
        .read_line(&mut user_response)
        .expect("Failed to read response");

    return user_response.trim().to_string();
}
