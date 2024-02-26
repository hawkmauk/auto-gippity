use crate::apis::call_request::call_gpt;
use crate::helpers::command_line::PrintCommand;
use crate::models::general::llm::Message;
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::fs;
use regex::Regex;

const CODE_TEMPLATE_PATH: &str = "/home/ubuntu/rust_autogpt/web_template/src/code_template.rs";
pub const WEB_SERVER_PROJECT_PATH: &str = "/home/ubuntu/rust_autogpt/web_template/";
const EXEC_MAIN_PATH: &str = "/home/ubuntu/rust_autogpt/web_template/src/main.rs";
const API_SCHEMA_PATH: &str = "/home/ubuntu/rust_autogpt/auto_gippity/schemas/api_schema.json";


// Extend ai function to encourage specific output
pub fn extend_ai_function(ai_func: fn(&str) -> &'static str, func_input: &str) -> Message {
    // run the ai function to get the return string
    let ai_function_str = ai_func(func_input);

    // extend the string to encourage only printing the output
    let msg: String = format!(
        "FUNCTION {}
    INSTRUCTION You are a function printer. You ONLY print the results of functions.
    Nothing else. No commentry. Here is the input to the function: {}.
    Print out what the function will return.",
        ai_function_str, func_input
    );

    Message {
        role: "system".to_string(),
        content: msg,
    }
}

// perform call to LLM GPT - decoded
pub async fn ai_task_request_decoded<T: DeserializeOwned>(
    msg_context: String,
    agent_position: &str,
    agent_operation: &str,
    function_pass: for<'a> fn(&'a str) -> &'static str,
) -> T {
    let llm_response: String =
        ai_task_request(msg_context, agent_position, agent_operation, function_pass).await;

    let decoded_response: T = serde_json::from_str(llm_response.as_str())
        .expect("failed to decode ai response from serde json");

    return decoded_response;
}

// get ai response without markdown code wrapper
pub async fn ai_task_request_without_markdown(
    msg_context: String,
    agent_position: &str,
    agent_operation: &str,
    function_pass: for<'a> fn(&'a str) -> &'static str,
) -> String {
    let llm_response: String =
        ai_task_request(msg_context, agent_position, agent_operation, function_pass).await;
    
    let markdown_regex: Regex = Regex::new(r"(^```.*(\r\n|\r|\n)|```\s*$)").unwrap();
    let llm_result = markdown_regex.replace_all( llm_response.as_ref(),"");
    return llm_result.to_string();
}

// perform call to LLM GPT
pub async fn ai_task_request(
    msg_context: String,
    agent_position: &str,
    agent_operation: &str,
    function_pass: for<'a> fn(&'a str) -> &'static str,
) -> String {
    let extended_msg: Message = extend_ai_function(function_pass, &msg_context);

    PrintCommand::AICall.print_agent_message(agent_position, agent_operation);

    // get LLM response
    let llm_result: Result<String, Box<dyn std::error::Error + Send>> =
        call_gpt(vec![extended_msg.clone()]).await;

    // return success or try again
    match llm_result {
        Ok(llm_response) => {
            // remove markdown from response
            llm_response
        },
        Err(_) => call_gpt(vec![extended_msg.clone()])
            .await
            .expect("Failed to call LLM"),
    }
}

// check whether url is valid
pub async fn check_status_code(client: &Client, url: &str) -> Result<u16, reqwest::Error> {
    let response: reqwest::Response = client.get(url).send().await?;
    Ok(response.status().as_u16())
}

// get code template
pub fn read_code_template_contents() -> String {
    let path: String = String::from(CODE_TEMPLATE_PATH);
    fs::read_to_string(path).expect("Failed to read code template")
}

// get executable code
pub fn read_exec_main_contents() -> String {
    let path: String = String::from(EXEC_MAIN_PATH);
    fs::read_to_string(path).expect("Failed to read code template")
}

// save new backend code
pub fn save_backend_code(contents: &String) {
    let path: String = String::from(EXEC_MAIN_PATH);
    fs::write(path, contents).expect("Failed to write main.rs")
}

// save the json api endpoint schema
pub fn save_api_endpoints(contents: &String) {
    let path: String = String::from(API_SCHEMA_PATH);
    fs::write(path, contents).expect("Failed to write api endpoints file")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_functions::aifunc_managing::convert_user_input_to_goal;

    #[test]
    fn tests_extending_ai_function() {
        let extended_message: Message =
            extend_ai_function(convert_user_input_to_goal, "dummy variable");
        assert_eq!(extended_message.role, "system".to_string());
    }

    #[tokio::test]
    async fn tests_ai_task_request() {
        let ai_func_param = "Build me a webserver that makes pigs fly".to_string();

        let res: String = ai_task_request(
            ai_func_param,
            "Managing Agent",
            "Defining user requirements",
            convert_user_input_to_goal,
        )
        .await;

        assert!(res.len() > 20);
        dbg!(res);
    }
}
