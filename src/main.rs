use regex::Regex;
use std::process::Command;
use ilo_hw_metrics::ilo_api_mod::IloSession;
use clap::{Arg, App};


#[derive(Debug)]
struct Ilo<'a> {
    ip_address: &'a str,
}

struct CommandIpmi<'a> {
    command: &'a str,
    arguments: Vec<&'a str>,
}

struct IloStatus(String);

impl IloStatus {
    fn build_ilo_params(&self) -> Ilo {
        let re = Regex::new(r".*(IP Address).*: (\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}).*").unwrap();
        let cap = re.captures(&self.0).unwrap();
        let ip = cap.get(2).unwrap().as_str();

        Ilo { ip_address: &ip }
    }
}

impl CommandIpmi<'_> {
    fn run() -> CommandIpmi<'static> {
        CommandIpmi {
            command: "ipmitool",
            arguments: vec!["lan", "print"],
        }
    }

    fn get_ilo_status(self) -> IloStatus {
        let ilo_status = Command::new(self.command)
            .args(self.arguments)
            .output()
            .expect("Cannot get ILO status.");

        let string_data = match std::string::String::from_utf8(ilo_status.stdout) {
            Ok(ip) => ip.to_string(),
            Err(_) => "Cannot get the IP address.".to_string(),
        };

        IloStatus(string_data)
    }
}

fn cli_parse() -> Result<clap::ArgMatches<'static>, ()> {
    let matches = App::new("HP Machine status")
        .version("0.1.0")
        .author("chama-chomo")
        .about("Retrieves HP machine status via ILO API")
        .arg(Arg::with_name("user")
                 .short("u")
                 .long("user")
                 .takes_value(true)
                 .help("ILO user"))
        .arg(Arg::with_name("passw")
                 .short("p")
                 .long("password")
                 .takes_value(true)
                 .help("ILO passwd"))
        .get_matches();


    Ok(matches)
}

fn main() {

    // pars cli args
    let args =  cli_parse().expect("Failed to obtain cli args");
    let user = args.value_of("user").unwrap_or("Administrator");
    let passwd = args.value_of("passw").unwrap();

    let ilo_status = CommandIpmi::run().get_ilo_status();
    let ilo_ip = ilo_status.build_ilo_params();

    // Create ILO session
    let url: &str =  &("https://".to_owned() + &ilo_ip.ip_address);
    let init_session = IloSession::create(url, &user, &passwd).expect("Token not acquired");

    let chassis_data = init_session.chassis().expect("Chassis data could not be obtained");

    println!("Chassis health: {:?}", &chassis_data)
}
