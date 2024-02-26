use crate::ai_functions::aifunc_backend::{
    print_backend_webserver_code, print_fixed_code, print_improved_webserver_code,
    print_rest_api_endpoints,
};
use crate::helpers::command_line::{ PrintCommand, confirm_safe_code };
use crate::helpers::general::{
    check_status_code, read_code_template_contents, read_exec_main_contents, save_api_endpoints,
    save_backend_code, ai_task_request_without_markdown, WEB_SERVER_PROJECT_PATH,
};
use crate::models::agent_basic::basic_agent::{AgentState, BasicAgent};
use crate::models::agents::agent_traits::{FactSheet, RouteObject, SpecialFunctions};

use crossterm::{
    style::{ SetForegroundColor, Color, ResetColor },
    ExecutableCommand,
};

use async_trait::async_trait;
use reqwest::Client;
use std::fs;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time;

#[derive(Debug)]
pub struct AgentBackendDeveloper {
    attributes: BasicAgent,
    bug_errors: Option<String>,
    bug_count: u8,
}

impl AgentBackendDeveloper {
    pub fn new() -> Self {
        let attributes: BasicAgent = BasicAgent {
            objective: "Develops the backend code for webserver and json database".to_string(),
            position: "Backend Developer".to_string(),
            state: AgentState::Discovery,
            memory: vec![],
        };

        Self {
            attributes,
            bug_errors: None,
            bug_count: 0,
        }
    }

    async fn call_initial_backend_code(&mut self, factsheet: &mut FactSheet) {
        let code_template_str = read_code_template_contents();

        let msg_context: String = format!(
            "CODE TEMPLATE: {} \n PROJECT_DESCRIPTION: {} \n
            OUTPUT IN PLAIN TEXT ONLY",
            code_template_str, factsheet.project_description
        );

        let ai_response: String = ai_task_request_without_markdown(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_backend_webserver_code),
            print_backend_webserver_code,
        )
        .await;

        save_backend_code(&ai_response);
        factsheet.backend_code = Some(ai_response);
    }

    async fn call_improved_backend_code(&mut self, factsheet: &mut FactSheet) {
        let msg_context: String = format!(
            "CODE TEMPLATE: {:?} \n PROJECT_DESCRIPTION: {:?} \n",
            factsheet.backend_code, factsheet
        );

        let ai_response: String = ai_task_request_without_markdown(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_improved_webserver_code),
            print_improved_webserver_code,
        )
        .await;

        save_backend_code(&ai_response);
        factsheet.backend_code = Some(ai_response);
    }

    async fn call_fix_code_bugs(&mut self, factsheet: &mut FactSheet) {
        let msg_context: String = format!(
            "BROKEN_CODE: {:?} \n ERROR_BUGS: {:?} \n",
            factsheet.backend_code, self.bug_count
        );

        let ai_response: String = ai_task_request_without_markdown(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_fixed_code),
            print_fixed_code,
        )
        .await;

        save_backend_code(&ai_response);
        factsheet.backend_code = Some(ai_response);
    }

    async fn call_extract_api_endpoints(&mut self) -> String {

        dbg!("call_extract_api_endpoints");
        let backend_code = read_exec_main_contents();

        let msg_context: String = format!("CODE_INPUT: {}", backend_code);

        let ai_response: String = ai_task_request_without_markdown(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_rest_api_endpoints),
            print_rest_api_endpoints,
        )
        .await;

        ai_response
    }
}

#[async_trait]
impl SpecialFunctions for AgentBackendDeveloper {
    fn get_attributes_from_agent(&self) -> &BasicAgent {
        &self.attributes
    }

    async fn execute(
        &mut self,
        factsheet: &mut FactSheet,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // !!! WARNING - BE CAREFUL OF INFINITE LOOPS !!!
        while self.attributes.state != AgentState::Finished {
            match self.attributes.state {

                AgentState::Discovery => {
                    self.call_initial_backend_code(factsheet).await;
                    self.attributes.state = AgentState::Working;
                    continue;
                }

                AgentState::Working => {
                    if self.bug_count == 0 {
                        self.call_improved_backend_code(factsheet).await;
                    } else {
                        self.call_fix_code_bugs(factsheet).await;
                    }

                    self.attributes.state = AgentState::UnitTesting;
                    continue;
                }

                AgentState::UnitTesting => {

                    //guard:  Ensure AI Safety
                    PrintCommand::UnitTest.print_agent_message(
                        self.attributes.position.as_str(),
                        "Backend code unit testing: ensuring safe code"
                    );

                    let is_safe_code = confirm_safe_code();

                    if ! is_safe_code {
                        panic!("Better go work on some AI alignment instead");
                    };

                    // build and test code
                    PrintCommand::UnitTest.print_agent_message(
                        self.attributes.position.as_str(),
                        "Backend code unit testing: building project..."
                    );

                    let build_backend_server: std::process::Output =
                        Command::new("cargo")
                        .arg("build")
                        .current_dir(WEB_SERVER_PROJECT_PATH)
                        .stdout(Stdio::piped())
                        .output()
                        .expect("Failed to build backed application");

                    // determine if build errors
                    if build_backend_server.status.success(){
                        self.bug_count = 0;
                        PrintCommand::UnitTest.print_agent_message(
                            self.attributes.position.as_str(),
                            "Backend code unit testing: build successful..."
                        );
                    } else {
                        let error_arr: Vec<u8> = build_backend_server.stderr;
                        let error_str: String = String::from_utf8(error_arr).unwrap();

                        // update error stats
                        self.bug_count += 1;
                        self.bug_errors = Some(error_str);

                        // exit if too many bugs
                        if self.bug_count > 2 {
                            PrintCommand::Issue.print_agent_message(
                                self.attributes.position.as_str(),
                                "Backend code unit testing: too many bugs found in code"
                            );
                            panic!("Error: Too many bugs");
                        }
                        
                        self.attributes.state = AgentState::Working;
                        continue;
                    }

                    /**
                     * Extract and Test
                     * Rest API Endpoints
                     */
                    let api_endpoints_str = self.call_extract_api_endpoints().await;

                    dbg!(api_endpoints_str.as_str());
                    //convert api endpoints into values
                    let api_endpoints: Vec<RouteObject> =
                        serde_json::from_str(api_endpoints_str.as_str())
                        .expect("Failed to decode API Endpoints");

                    let check_endpoints: Vec<RouteObject> = api_endpoints
                        .iter()
                        .filter(|&route_object| {
                            route_object.method == "get" && route_object.is_route_dynamic == "false"
                        })
                        .cloned()
                        .collect();

                    //store api endpoints
                    factsheet.api_endpoint_schema = Some(check_endpoints.clone());

                    // run backed application
                    PrintCommand::UnitTest.print_agent_message(
                        self.attributes.position.as_str(),
                        "Backend code unit testing: starting web server..."
                    );

                    let mut run_backend_server: std::process::Child =
                        Command::new("cargo")
                        .arg("run")
                        .current_dir(WEB_SERVER_PROJECT_PATH)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("Failed to run backed application");

                    // let user know testing on server will take place soon
                    PrintCommand::UnitTest.print_agent_message(
                        self.attributes.position.as_str(),
                        "Backend code unit testing: launching tests on server in 5s..."
                    );

                    let seconds_sleep: Duration = Duration::from_secs(5);
                    time::sleep( seconds_sleep ).await;

                    // check status code
                    for endpoint in check_endpoints {
                        
                        let testing_msg: String = format!("Testing endpoint '{}'...", endpoint.route);
                        PrintCommand::UnitTest.print_agent_message(
                            self.attributes.position.as_str(),
                            testing_msg.as_str()
                        );

                        // create client with timeout
                        let client : Client = Client::builder()
                            .timeout(Duration::from_secs(5))
                            .build()
                            .unwrap();

                        // test url
                        let url_string = format!("http://localhost:6678{}", endpoint.route);
                        match check_status_code(&client, &url_string).await {

                            Ok(status_code) => {
                                if status_code != 200 {
                                    let error_msg: String = format!("WARNING: Failed to call backend url endpoint {}", endpoint.route);
                                    PrintCommand::Issue.print_agent_message(
                                        self.attributes.position.as_str(),
                                        error_msg.as_str()
                                    );

                                }
                            }

                            Err( e ) => {
                                run_backend_server
                                    .kill()
                                    .expect("Failed to kill backend web server");

                                let error_msg: String = format!("Errro checking backend {}", e );
                                
                                PrintCommand::Issue.print_agent_message(
                                    self.attributes.position.as_str(),
                                    error_msg.as_str()
                                )
                            }
                        }

                    }

                    save_api_endpoints( &api_endpoints_str );

                    PrintCommand::UnitTest.print_agent_message(
                        self.attributes.position.as_str(),
                        "Backend testing complete..."
                    );

                    run_backend_server
                        .kill()
                        .expect("Failed to kill backend web server on comp[letion");

                    self.attributes.state = AgentState::Finished;
                }

                _ => {
                    self.attributes.state = AgentState::Finished;
                }
            }
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn tests_writing_backend_code() {
        let mut agent = AgentBackendDeveloper::new();

        let factsheet_str: &str = r#"{
        "project_description": "build a website with user login and logout that stores addresses and phone numbers of contacts",
        "project_scope": {
            "is_crud_required": true,
            "is_user_login_and_logout": true,
            "is_external_urls_required": true
        },
        "external_urls": [
            "http://worldtimeapi.org/api/timezone"
        ],
        "backend_code": null,
            "api_endpoint_schema": null
        }"#;

        let mut factsheet: FactSheet = serde_json::from_str(factsheet_str).unwrap();

//        agent.attributes.state = AgentState::UnitTesting;
        agent
            .execute(&mut factsheet)
            .await
            .expect("Failed to execute backend  developer agent");

        dbg!(factsheet);
    }
}
