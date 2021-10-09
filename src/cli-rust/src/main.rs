use {
    borsh::BorshDeserialize,
    clap::{crate_description, crate_name, crate_version, value_t, App, Arg},
    helloworld::{instruction::init_greeting, processor::GreetingAccount},
    solana_clap_utils::{
        input_parsers::{keypair_of, value_of},
        input_validators::{is_amount, is_keypair_or_ask_keyword},
        keypair::signer_from_path,
    },
    solana_client::rpc_client::RpcClient,
    solana_program::{native_token::lamports_to_sol, program_pack::Pack, pubkey::Pubkey},
    solana_sdk::{
        commitment_config::CommitmentConfig, signature::Signer,
        system_instruction::create_account_with_seed, transaction::Transaction,
    },
    std::process::exit,
};

struct Config {
    rpc_client: RpcClient,
    fee_payer: Box<dyn Signer>,
    program_id: Pubkey,
    dry_run: bool,
}

type Error = Box<dyn std::error::Error>;
type CommandResult = Result<(), Error>;

const PROGRAM_ID: &str = "../../dist/program/helloworld-keypair.json";

fn main() -> CommandResult {
    solana_logger::setup_with_default("solana=info");

    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        // .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg({
            let arg = Arg::with_name("config_file")
                .short("C")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("program_id")
                .long("program")
                .validator(is_keypair_or_ask_keyword)
                .value_name("PROG_KEYPAIR_PATH")
                .takes_value(true)
                .required(false)
                .default_value(PROGRAM_ID)
                .help("program ID"),
        )
        .arg(
            Arg::with_name("greeting_account_seed")
                .long("seed")
                .value_name("SEED")
                .takes_value(true)
                .required(false)
                .default_value("hello")
                .help("greeting account seed"),
        )
        .arg(
            Arg::with_name("greetings")
                .long("greetings")
                .validator(is_amount)
                .value_name("GREETINGS")
                .takes_value(true)
                .required(false)
                .default_value("1")
                .help("number of greetings to increment"),
        )
        .arg(
            Arg::with_name("greeting_string")
                .long("greeting_string")
                .value_name("GREETING_STRING")
                .takes_value(true)
                .required(false)
                .default_value("hello")
                .help("Extra greeting string to pass"),
        )
        .arg(
            Arg::with_name("dry_run")
                .long("dry-run")
                .takes_value(false)
                .global(true)
                .help("Simulate transaction instead of executing"),
        )
        .get_matches();

    let mut wallet_manager = None;
    let config = {
        let cli_config = if let Some(config_file) = matches.value_of("config_file") {
            solana_cli_config::Config::load(config_file).unwrap_or_default()
        } else {
            solana_cli_config::Config::default()
        };
        let json_rpc_url = value_t!(matches, "json_rpc_url", String)
            .unwrap_or_else(|_| cli_config.json_rpc_url.clone());

        let fee_payer = signer_from_path(
            &matches,
            matches
                .value_of("fee_payer")
                .unwrap_or(&cli_config.keypair_path),
            "fee_payer",
            &mut wallet_manager,
        )
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(1);
        });

        let program_id = keypair_of(&matches, "program_id").unwrap().pubkey();
        let dry_run = matches.is_present("dry_run");

        Config {
            rpc_client: RpcClient::new_with_commitment(json_rpc_url, CommitmentConfig::confirmed()),
            fee_payer,
            program_id,
            dry_run,
        }
    };
    // Parse inputs from arguments
    let num_greetings: u32 = value_of(&matches, "greetings").unwrap();
    let greeting_string = String::from(matches.value_of("greeting_string").unwrap());

    let greeting_account_seed = matches.value_of("greeting_account_seed").unwrap();

    let greeting_account_id = Pubkey::create_with_seed(
        &config.fee_payer.pubkey(),
        greeting_account_seed,
        &config.program_id,
    )?;

    if let Ok(_) = config.rpc_client.get_account(&greeting_account_id) {
        println!("greeting account {} exists!", &greeting_account_id);
    } else {
        println!(
            "greeting account {} does not exist. Let's create it!",
            &greeting_account_id
        );

        let lamports = config
            .rpc_client
            .get_minimum_balance_for_rent_exemption(GreetingAccount::LEN)?;
        let mut transaction = Transaction::new_with_payer(
            &[create_account_with_seed(
                &config.fee_payer.pubkey(),
                &greeting_account_id,
                &config.fee_payer.pubkey(),
                &greeting_account_seed,
                lamports,
                GreetingAccount::LEN as u64,
                &config.program_id,
            )],
            Some(&config.fee_payer.pubkey()),
        );

        let (recent_blockhash, fee_calculator) = config.rpc_client.get_recent_blockhash()?;
        check_fee_payer_balance(
            &config,
            lamports + fee_calculator.calculate_fee(transaction.message()),
        )?;
        transaction.sign(&vec![config.fee_payer.as_ref()], recent_blockhash);
        send_transaction(&config, transaction)?;
    };

    //Create greeting transaction
    let mut transaction = Transaction::new_with_payer(
        &[init_greeting(
            config.program_id,
            greeting_account_id,
            num_greetings,
            greeting_string,
        )],
        Some(&config.fee_payer.pubkey()),
    );

    // Check fee balance, sign and send
    let (recent_blockhash, fee_calculator) = config.rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(&config, fee_calculator.calculate_fee(transaction.message()))?;
    transaction.sign(&vec![config.fee_payer.as_ref()], recent_blockhash);
    println!("Adding {} greetings...", &num_greetings);
    send_transaction(&config, transaction)?;

    // Report on number of greetings
    let data = config.rpc_client.get_account_data(&greeting_account_id)?;
    let greeting_account = GreetingAccount::try_from_slice(&data)?;
    println!("Greeted {} time(s)!", greeting_account.counter);
    println!(
        "Greetings times 2 equals {}!",
        greeting_account.counter_times_2
    );
    Ok(())
}

// HELPERS

fn send_transaction(
    config: &Config,
    transaction: Transaction,
) -> solana_client::client_error::Result<()> {
    if config.dry_run {
        let result = config.rpc_client.simulate_transaction(&transaction)?;
        println!("Simulate result: {:?}", result);
    } else {
        let signature = config
            .rpc_client
            .send_and_confirm_transaction_with_spinner(&transaction)?;
        println!("Signature: {}", signature);
    }
    Ok(())
}
fn check_fee_payer_balance(config: &Config, required_balance: u64) -> Result<(), Error> {
    let balance = config.rpc_client.get_balance(&config.fee_payer.pubkey())?;
    if balance < required_balance {
        Err(format!(
            "Fee payer, {}, has insufficient balance: {} required, {} available",
            config.fee_payer.pubkey(),
            lamports_to_sol(required_balance),
            lamports_to_sol(balance)
        )
        .into())
    } else {
        Ok(())
    }
}
