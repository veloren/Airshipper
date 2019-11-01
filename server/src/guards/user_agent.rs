use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

lazy_static::lazy_static! {
    static ref AIRSHIPPER_AGENT: regex::Regex = regex::Regex::new(r#"Airshipper/(?P<version>[\S]+) \((?P<platform>.+)\)"#).expect("compiling regex failed!");
}

/// This Request Guard ensures that the request was made from the Airshipper UserAgent.
#[derive(Debug)]
pub struct AirshipperAgent {
    platform: Option<String>,
    version: Option<String>,
}

#[derive(Debug)]
pub enum AgentError {
    InvalidAgent,
    MultipleAgents,
}

impl<'a, 'r> FromRequest<'a, 'r> for AirshipperAgent {
    type Error = AgentError;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("User-Agent").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::BadRequest, AgentError::InvalidAgent)),
            1 => {
                // UserAgent: Airshipper/<VERSION> (<PLATFORM>)
                if keys[0].starts_with("Airshipper") {
                    if let Some(caps) = AIRSHIPPER_AGENT.captures(keys[0]) {
                        let platform = caps.name("platform").map(|x| x.as_str().to_string());
                        let version = caps.name("version").map(|x| x.as_str().to_string());

                        Outcome::Success(AirshipperAgent { platform, version })
                    } else {
                        Outcome::Success(AirshipperAgent {
                            platform: None,
                            version: None,
                        })
                    }
                } else {
                    Outcome::Forward(())
                }
            }
            _ => Outcome::Failure((Status::BadRequest, AgentError::MultipleAgents)),
        }
    }
}

/// This Request Guard ensures that the request was NOT made from the Airshipper UserAgent.
#[derive(Debug)]
pub struct NonAirshipperAgent(Option<String>);

#[derive(Debug)]
pub enum NonAgentError {
    MultipleAgents,
}

impl<'a, 'r> FromRequest<'a, 'r> for NonAirshipperAgent {
    type Error = NonAgentError;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("User-Agent").collect();
        match keys.len() {
            0 => Outcome::Success(NonAirshipperAgent(None)),
            1 => {
                if !keys[0].starts_with("Airshipper") {
                    Outcome::Success(NonAirshipperAgent(Some(keys[0].to_string())))
                } else {
                    Outcome::Forward(())
                }
            }
            _ => Outcome::Failure((Status::BadRequest, NonAgentError::MultipleAgents)),
        }
    }
}
