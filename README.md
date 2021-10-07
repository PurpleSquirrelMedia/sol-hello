<p align="center">
  <a href="https://solana.com">
    <img alt="Solana" src="https://i.imgur.com/uBVzyX3.png" width="250" />
  </a>
</p>

[![build status](https://github.com/CalebEverett/example-helloworld/actions/workflows/main.yml/badge.svg)](https://github.com/CalebEverett/example-helloworld/actions/workflows/main.yml)[![Gitpod
Ready-to-Code](https://img.shields.io/badge/Gitpod-Ready--to--Code-blue?logo=gitpod)](https://gitpod.io/#https://github.com/CalebEverett/example-helloworld)


# Hello world on Solana - extended

This is an extended version of the official Solana [example-helloworld](https://github.com/solana-labs/example-helloworld) project.

The original project included:

* An on-chain hello world program
* A client that can send a "hello" to an account and get back the number of
  times "hello" has been sent

This extended version adds the following functionality:

* The on-chain program processes serialized input for the number of greetings to
  add to the counter.
* The on chain program adds another field to the Greeting Account, which trivially
  stores the result of 2 * the original counter field
* The javascript client serializes and sends a set number to be processed by the on
  chain program
* Includes a rust based command line program to interact with the on chain program, including
  * Creating a greeting account of the appropriate size - to accommodate both fields (counter and counter_times _2)
  * Accepts an argument for the number of greetings to send

## Getting Started

1. Follow the instructions in original [README.md](README_original.md) to

  * Install Node, Rust and Solana installed
  * Install node dependencies
  * Start the local test validator and Solana logs
  * Build and deploy the on chain program
  
2. At this point you you should be able to run the javascript client with `npm run start` and
    see that a greeting account has been created on chain and that the number of greetings 
    has been incremented

3. To interact with the Rust cli program, first build it with `npm run build:cli-rust`

4. Then change into the `src/cli-rust` directory and run `cargo run -- --help` to see the
    the available command line arguments. To increment by one, you can run `cargo run`.


## Motivation

The original [example-helloworld](https://github.com/solana-labs/example-helloworld) project was a great
starting place. This extended example continues the learning with added functionality to:

* Pass data to on chain programs
* Interact with on chain programs from Rust command line programs

As a newcomer to Rust and Solana with intermediate Python experience, working through the details of implementing these features helped to better understand how Solana works. [The programming model docs](https://docs.solana.com/developing/programming-model/overview) explain all this more precisely, in more detail, but here is a high level overview.

1. Programs live on chain, but don't store any data.

2. Data is stored in separate accounts that can be accessed by programs.

3. All data is stored as bytes.

4. The process of turning data that you use in your program into bytes to be
  stored is called serialization. The process of decoding stored bytes into
  data you use in your programs is called deserialization. In super layman's terms
  you basically have a series of 8 bit numbers (up to 255 for each) that get stored in an
  array in storage. The process of serializing and deserializing entails knowing how those bytes
  relate to the data in your program. In this extended example, we store two unsigned 32 bit numbers, which means that we have an array of 8 bits where we have decided in our program that the first four
  relate to `counter` and the second four relate to `counter_times_2`. Check out [processors.rs](src/program-rust/src/processor.rs) to see how that works.

5. Same thing as it relates to passing data into your programs. You have to serialize the data that
  you are going to pass in and then add logic to your program to deserialize it. Check out the `sayHello` function in [helloworld.ts](src/client/hello_world.ts) and [instruction.rs](src/program-rust/src/instruction.rs) to see how it works on the client side. Note that that instruction includes serialization and deserialization (packing and unpacking) methods that get used by both the onchain and Rust cli programs.

6. You interact with on chain programs by sending it [Transactions](src/program-rust/src/instruction.rs), comprised of one or more [Instructions](https://docs.rs/solana-program/1.8.0/solana_program/instruction/struct.Instruction.html). Instructions are just the program that it should be sent to, a list of accounts the program uses when it processes the instruction and then the data the program needs to process the instruction. Again, that data is just bytes and you have to have set up the serialization on the client side and deserialization on the program side so that they match. This is greatly facilitated by using the same Instruction construct in both the client and onchain programs.

7. This program only has a single instruction, but you can process different instructions by using the first bit of the Instruction data that gets passed in to identify the instruction type. If you look in [instruction.rs](src/program-rust/src/instruction.rs), you can see that the first
bit is matched to return a corresponding instruction type from the instruction enum.

8 . Given the flow of a program:
  * Receive serialized instruction
  * Determine type of instruction received based on first bit of deserialized data
  * Process instruction based on instruction type<br> it makes sense that many programs separate concerns by having different modules for instructions and processors. Other programs seem to also separate out errors and state. You can look at the [token-lending-program](https://github.com/solana-labs/solana-program-library/tree/master/token-lending/program/src) in the Solana Program Library to see a fully developed program.

  ## Other Things Worth Checking Out