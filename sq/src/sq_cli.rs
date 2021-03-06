/// Command-line parser for sq.
///
/// If you change this file, please rebuild `sq`, run `make -C tool
/// update-usage`, and commit the resulting changes to
/// `sq/src/sq-usage.rs`.

use clap::{App, Arg, ArgGroup, SubCommand, AppSettings};

pub fn build() -> App<'static, 'static> {
    App::new("sq")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Sequoia is an implementation of OpenPGP.  This is a command-line frontend.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("home").value_name("DIRECTORY")
             .long("home")
             .help("Sets the home directory to use"))
        .arg(Arg::with_name("mapping").value_name("MAPPING")
             .long("mapping")
             .short("m")
             .default_value("org.sequoia-pgp.contacts/default")
             .help("Sets the realm and mapping to use"))
        .arg(Arg::with_name("policy").value_name("NETWORK-POLICY")
             .long("policy")
             .short("p")
             .help("Sets the network policy to use"))
        .arg(Arg::with_name("force")
             .long("force")
             .short("f")
             .help("Overwrite existing files"))
        .arg(Arg::with_name("known-notation")
             .long("known-notation")
             .multiple(true)
             .takes_value(true)
             .value_name("NOTATION")
             .number_of_values(1)
             .help("The notation name is considered known. \
               This is used when validating signatures. \
               Signatures that have unknown notations with the \
               critical bit set are considered invalid."))
        .subcommand(SubCommand::with_name("decrypt")
                    .display_order(10)
                    .about("Decrypts an OpenPGP message")
                    .arg(Arg::with_name("input").value_name("FILE")
                         .help("Sets the input file to use"))
                    .arg(Arg::with_name("output").value_name("FILE")
                         .long("output")
                         .short("o")
                         .help("Sets the output file to use"))
                    .arg(Arg::with_name("signatures").value_name("N")
                         .help("The number of valid signatures required.  \
                                Default: 0")
                         .long("signatures")
                         .short("n")
                         .takes_value(true))
                    .arg(Arg::with_name("sender-cert-file")
                         .long("sender-cert-file")
                         .multiple(true)
                         .takes_value(true)
                         .value_name("CERT-FILE")
                         .number_of_values(1)
                         .help("The sender's certificate verify signatures \
                                with, given as a file \
                                (can be given multiple times)"))
                    .arg(Arg::with_name("secret-key-file")
                         .long("secret-key-file")
                         .multiple(true)
                         .takes_value(true)
                         .value_name("TSK-FILE")
                         .number_of_values(1)
                         .help("Secret key to decrypt with, given as a file \
                                (can be given multiple times)"))
                    .arg(Arg::with_name("dump-session-key")
                         .long("dump-session-key")
                         .help("Prints the session key to stderr"))
                    .arg(Arg::with_name("dump")
                         .long("dump")
                         .help("Print a packet dump to stderr"))
                    .arg(Arg::with_name("hex")
                         .long("hex")
                         .short("x")
                         .help("Print a hexdump (implies --dump)")))
        .subcommand(SubCommand::with_name("encrypt")
                    .display_order(20)
                    .about("Encrypts a message")
                    .arg(Arg::with_name("input").value_name("FILE")
                         .help("Sets the input file to use"))
                    .arg(Arg::with_name("output").value_name("FILE")
                         .long("output")
                         .short("o")
                         .help("Sets the output file to use"))
                    .arg(Arg::with_name("binary")
                         .long("binary")
                         .short("B")
                         .help("Don't ASCII-armor encode the OpenPGP data"))
                    .arg(Arg::with_name("recipient")
                         .long("recipient")
                         .short("r")
                         .multiple(true)
                         .takes_value(true)
                         .value_name("LABEL")
                         .number_of_values(1)
                         .help("Recipient to encrypt for \
                                (can be given multiple times)"))
                    .arg(Arg::with_name("recipients-cert-file")
                         .long("recipients-cert-file")
                         .multiple(true)
                         .takes_value(true)
                         .value_name("CERTS-FILE")
                         .number_of_values(1)
                         .help("Recipients to encrypt for, given as a file \
                                (can be given multiple times)"))
                    .arg(Arg::with_name("signer-key-file")
                         .long("signer-key-file")
                         .multiple(true)
                         .takes_value(true)
                         .value_name("TSK-FILE")
                         .number_of_values(1)
                         .help("Secret key to sign with, given as a file \
                                (can be given multiple times)"))
                    .arg(Arg::with_name("symmetric")
                         .long("symmetric")
                         .short("s")
                         .multiple(true)
                         .help("Encrypt with a password \
                                (can be given multiple times)"))
                    .arg(Arg::with_name("mode").value_name("MODE")
                         .long("mode")
                         .possible_values(&["transport", "rest", "all"])
                         .default_value("all")
                         .help("Selects what kind of keys are considered for \
                                encryption.  Transport select subkeys marked \
                                as suitable for transport encryption, rest \
                                selects those for encrypting data at rest, \
                                and all selects all encryption-capable \
                                subkeys"))
                    .arg(Arg::with_name("compression")
                         .value_name("KIND")
                         .long("compression")
                         .possible_values(&["none", "pad", "zip", "zlib",
                                            "bzip2"])
                         .default_value("pad")
                         .help("Selects compression scheme to use"))
                    .arg(Arg::with_name("time").value_name("TIME")
                         .long("time")
                         .short("t")
                         .help("Chooses keys valid at the specified time and \
                                sets the signature's creation time"))
                    .arg(Arg::with_name("use-expired-subkey")
                         .long("use-expired-subkey")
                         .help("If a certificate has only expired \
                                encryption-capable subkeys, fall back \
                                to using the one that expired last"))
        )

        .subcommand(SubCommand::with_name("sign")
                    .display_order(25)
                    .about("Signs a message")
                    .arg(Arg::with_name("input").value_name("FILE")
                         .help("Sets the input file to use"))
                    .arg(Arg::with_name("output").value_name("FILE")
                         .long("output")
                         .short("o")
                         .help("Sets the output file to use"))
                    .arg(Arg::with_name("binary")
                         .long("binary")
                         .short("B")
                         .help("Don't ASCII-armor encode the OpenPGP data"))
                    .arg(Arg::with_name("detached")
                         .long("detached")
                         .help("Create a detached signature"))
                    .arg(Arg::with_name("append")
                         .long("append")
                         .short("a")
                         .conflicts_with("notarize")
                         .help("Append signature to existing signature"))
                    .arg(Arg::with_name("notarize")
                         .long("notarize")
                         .short("n")
                         .conflicts_with("append")
                         .help("Signs a message and all existing signatures"))
                    .arg(Arg::with_name("secret-key-file")
                         .long("secret-key-file")
                         .multiple(true)
                         .takes_value(true)
                         .value_name("TSK-FILE")
                         .number_of_values(1)
                         .help("Secret key to sign with, given as a file \
                                (can be given multiple times)"))
                    .arg(Arg::with_name("time").value_name("TIME")
                         .long("time")
                         .short("t")
                         .help("Chooses keys valid at the specified time and \
                                sets the signature's creation time")))
        .subcommand(SubCommand::with_name("verify")
                    .display_order(26)
                    .about("Verifies a message")
                    .arg(Arg::with_name("input").value_name("FILE")
                         .help("Sets the input file to use"))
                    .arg(Arg::with_name("output").value_name("FILE")
                         .long("output")
                         .short("o")
                         .help("Sets the output file to use"))
                    .arg(Arg::with_name("detached")
                         .long("detached")
                         .takes_value(true)
                         .value_name("SIG-FILE")
                         .help("Verifies a detached signature"))
                    .arg(Arg::with_name("signatures").value_name("N")
                         .help("The number of valid signatures required.  \
                                Default: 0")
                         .long("signatures")
                         .short("n")
                         .takes_value(true))
                    .arg(Arg::with_name("sender-cert-file")
                         .long("sender-cert-file")
                         .multiple(true)
                         .takes_value(true)
                         .value_name("CERT-FILE")
                         .number_of_values(1)
                         .help("The sender's certificate verify signatures \
                                with, given as a file \
                                (can be given multiple times)")))
        .subcommand(SubCommand::with_name("enarmor")
                    .about("Applies ASCII Armor to a file")
                    .arg(Arg::with_name("input").value_name("FILE")
                         .help("Sets the input file to use"))
                    .arg(Arg::with_name("output").value_name("FILE")
                         .long("output")
                         .short("o")
                         .help("Sets the output file to use"))
                    .arg(Arg::with_name("kind")
                         .value_name("KIND")
                         .long("kind")
                         .possible_values(&["message", "publickey", "secretkey",
                                            "signature", "file"])
                         .default_value("file")
                         .help("Selects the kind of header line to produce")))

        .subcommand(SubCommand::with_name("dearmor")
                    .about("Removes ASCII Armor from a file")
                    .arg(Arg::with_name("input").value_name("FILE")
                         .help("Sets the input file to use"))
                    .arg(Arg::with_name("output").value_name("FILE")
                         .long("output")
                         .short("o")
                         .help("Sets the output file to use")))
        .subcommand(SubCommand::with_name("autocrypt")
                    .about("Autocrypt support")
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .subcommand(SubCommand::with_name("decode")
                                .about("Converts Autocrypt-encoded keys to \
                                        OpenPGP Certificates")
                                .arg(Arg::with_name("input").value_name("FILE")
                                     .help("Sets the input file to use"))
                                .arg(Arg::with_name("output").value_name("FILE")
                                     .long("output")
                                     .short("o")
                                     .help("Sets the output file to use")))
                    .subcommand(SubCommand::with_name("encode-sender")
                                .about("Encodes the sender's OpenPGP \
                                        Certificates into \
                                        an Autocrypt header")
                                .arg(Arg::with_name("input").value_name("FILE")
                                     .help("Sets the input file to use"))
                                .arg(Arg::with_name("output").value_name("FILE")
                                     .long("output")
                                     .short("o")
                                     .help("Sets the output file to use"))
                                .arg(Arg::with_name("address")
                                     .long("address")
                                     .takes_value(true)
                                     .help("Select userid to use.  \
                                            [default: primary userid]"))
                                .arg(Arg::with_name("prefer-encrypt")
                                     .long("prefer-encrypt")
                                     .possible_values(&["nopreference",
                                                        "mutual"])
                                     .default_value("nopreference")
                                     .help("Sets the prefer-encrypt \
                                            attribute"))))
        .subcommand(SubCommand::with_name("inspect")
                    .about("Inspects a sequence of OpenPGP packets")
                    .arg(Arg::with_name("input").value_name("FILE")
                         .help("Sets the input file to use"))
                    .arg(Arg::with_name("keygrips")
                         .long("keygrips")
                         .help("Print keygrips of keys and subkeys"))
                    .arg(Arg::with_name("certifications")
                         .long("certifications")
                         .help("Print third-party certifications")))

        .subcommand(SubCommand::with_name("keyserver")
                    .display_order(40)
                    .about("Interacts with keyservers")
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .arg(Arg::with_name("server").value_name("URI")
                         .long("server")
                         .short("s")
                         .help("Sets the keyserver to use"))
                    .subcommand(SubCommand::with_name("get")
                                .about("Retrieves a key")
                                .arg(Arg::with_name("output").value_name("FILE")
                                     .long("output")
                                     .short("o")
                                     .help("Sets the output file to use"))
                                .arg(Arg::with_name("binary")
                                     .long("binary")
                                     .short("B")
                                     .help("Don't ASCII-armor encode the OpenPGP data"))
                                .arg(Arg::with_name("query").value_name("QUERY")
                                     .required(true)
                                     .help(
                                         "Fingerprint, KeyID, or email \
                                          address of the cert(s) to retrieve"
                                     )))
                    .subcommand(SubCommand::with_name("send")
                                .about("Sends a key")
                                .arg(Arg::with_name("input").value_name("FILE")
                                     .help("Sets the input file to use"))))
        .subcommand(SubCommand::with_name("mapping")
                    .display_order(30)
                    .about("Interacts with key mappings")
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .subcommand(SubCommand::with_name("list")
                                .about("Lists keys in the mapping"))
                    .subcommand(SubCommand::with_name("add")
                                .about("Add a key identified by fingerprint")
                                .arg(Arg::with_name("label").value_name("LABEL")
                                     .required(true)
                                     .help("Label to use"))
                                .arg(Arg::with_name("fingerprint").value_name("FINGERPRINT")
                                     .required(true)
                                     .help("Key to add")))
                    .subcommand(SubCommand::with_name("import")
                                .about("Imports a key")
                                .arg(Arg::with_name("label").value_name("LABEL")
                                     .required(true)
                                     .help("Label to use"))
                                .arg(Arg::with_name("input").value_name("FILE")
                                     .help("Sets the input file to use")))
                    .subcommand(SubCommand::with_name("export")
                                .about("Exports a key")
                                .arg(Arg::with_name("label").value_name("LABEL")
                                     .required(true)
                                     .help("Label to use"))
                                .arg(Arg::with_name("output").value_name("FILE")
                                     .long("output")
                                     .short("o")
                                     .help("Sets the output file to use"))
                                .arg(Arg::with_name("binary")
                                     .long("binary")
                                     .short("B")
                                     .help("Don't ASCII-armor encode the OpenPGP data")))
                    .subcommand(SubCommand::with_name("delete")
                                .about("Deletes bindings or mappings")
                                .arg(Arg::with_name("the-mapping")
                                     .long("the-mapping")
                                     .help("Delete the selected mapping (change with --mapping)"))
                                .arg(Arg::with_name("label")
                                     .value_name("LABEL")
                                     .help("Delete binding with this label")))
                    .subcommand(SubCommand::with_name("stats")
                                .about("Get stats for the given label")
                                .arg(Arg::with_name("label").value_name("LABEL")
                                     .required(true)
                                     .help("Label to use")))
                    .subcommand(SubCommand::with_name("log")
                                .about("Lists the keystore log")
                                .arg(Arg::with_name("label")
                                     .value_name("LABEL")
                                     .help("List messages related to this label"))))
        .subcommand(SubCommand::with_name("list")
                    .about("Lists key mappings and known keys")
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .subcommand(SubCommand::with_name("mappings")
                                .about("Lists key mappings")
                                .arg(Arg::with_name("prefix").value_name("PREFIX")
                                     .help("List only mappings with the given realm prefix")))
                    .subcommand(SubCommand::with_name("bindings")
                                .about("Lists all bindings in all key mappings")
                                .arg(Arg::with_name("prefix").value_name("PREFIX")
                                     .help("List only bindings from mappings with the given realm prefix")))
                    .subcommand(SubCommand::with_name("keys")
                                .about("Lists all keys in the common key pool"))
                    .subcommand(SubCommand::with_name("log")
                                .about("Lists the server log")))
        .subcommand(
            SubCommand::with_name("key")
                .about("Manipulates keys")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("generate")
                        .about("Generates a new key")
                        .arg(Arg::with_name("userid")
                             .value_name("EMAIL")
                             .long("userid")
                             .short("u")
                             .multiple(true)
                             .number_of_values(1)
                             .takes_value(true)
                             .help("Add userid to the key \
                                    (can be given multiple times)"))
                        .arg(Arg::with_name("cipher-suite")
                             .value_name("CIPHER-SUITE")
                             .long("cipher-suite")
                             .short("c")
                             .possible_values(&["rsa3k", "rsa4k", "cv25519"])
                             .default_value("cv25519")
                             .help("Cryptographic algorithms used for the key."))
                        .arg(Arg::with_name("with-password")
                             .long("with-password")
                             .help("Prompt for a password to protect the \
                                    generated key with."))

                        .group(ArgGroup::with_name("expiration-group")
                               .args(&["expires", "expires-in"]))

                        .arg(Arg::with_name("expires")
                             .value_name("TIME")
                             .long("expires")
                             .help("Absolute time When the key should expire, \
                                    or 'never'."))
                        .arg(Arg::with_name("expires-in")
                             .value_name("DURATION")
                             .long("expires-in")
                             // Catch negative numbers.
                             .allow_hyphen_values(true)
                             .help("Relative time when the key should expire.  \
                                    Either 'N[ymwd]', for N years, months, \
                                    weeks, or days, or 'never'."))

                        .group(ArgGroup::with_name("cap-sign")
                               .args(&["can-sign", "cannot-sign"]))
                        .arg(Arg::with_name("can-sign")
                             .long("can-sign")
                             .help("The key has a signing-capable subkey \
                                    (default)"))
                        .arg(Arg::with_name("cannot-sign")
                             .long("cannot-sign")
                             .help("The key will not be able to sign data"))

                        .group(ArgGroup::with_name("cap-encrypt")
                               .args(&["can-encrypt", "cannot-encrypt"]))
                        .arg(Arg::with_name("can-encrypt").value_name("PURPOSE")
                             .long("can-encrypt")
                             .possible_values(&["transport", "storage",
                                                "universal"])
                             .help("The key has an encryption-capable subkey \
                                    (default: universal)"))
                        .arg(Arg::with_name("cannot-encrypt")
                             .long("cannot-encrypt")
                             .help("The key will not be able to encrypt data"))

                        .arg(Arg::with_name("export").value_name("OUTFILE")
                             .long("export")
                             .short("e")
                             .help("Exports the key instead of saving it in \
                                    the store")
                             .required(true))
                        .arg(Arg::with_name("rev-cert").value_name("FILE or -")
                             .long("rev-cert")
                             .required_if("export", "-")
                             .help("Sets the output file for the revocation \
                                    certificate. Default is <OUTFILE>.rev, \
                                    mandatory if OUTFILE is '-'."))))

        .subcommand(SubCommand::with_name("packet")
                    .about("OpenPGP Packet manipulation")
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .subcommand(SubCommand::with_name("dump")
                                .about("Lists OpenPGP packets")
                                .arg(Arg::with_name("input").value_name("FILE")
                                     .help("Sets the input file to use"))
                                .arg(Arg::with_name("output").value_name("FILE")
                                     .long("output")
                                     .short("o")
                                     .help("Sets the output file to use"))
                                .arg(Arg::with_name("session-key")
                                     .long("session-key")
                                     .takes_value(true)
                                     .value_name("SESSION-KEY")
                                     .help("Session key to decrypt encryption \
                                            containers"))
                                .arg(Arg::with_name("mpis")
                                     .long("mpis")
                                     .help("Print MPIs"))
                                .arg(Arg::with_name("hex")
                                     .long("hex")
                                     .short("x")
                                     .help("Print a hexdump")))

                    .subcommand(SubCommand::with_name("decrypt")
                                .display_order(10)
                                .about("Decrypts an OpenPGP message, dumping \
                                        the content of the encryption \
                                        container without further processing")
                                .arg(Arg::with_name("input").value_name("FILE")
                                     .help("Sets the input file to use"))
                                .arg(Arg::with_name("output").value_name("FILE")
                                     .long("output")
                                     .short("o")
                                     .help("Sets the output file to use"))
                                .arg(Arg::with_name("binary")
                                     .long("binary")
                                     .short("B")
                                     .help("Don't ASCII-armor encode the \
                                            OpenPGP data"))
                                .arg(Arg::with_name("secret-key-file")
                                     .long("secret-key-file")
                                     .multiple(true)
                                     .takes_value(true)
                                     .value_name("TSK-FILE")
                                     .number_of_values(1)
                                     .help("Secret key to decrypt with, given \
                                            as a file \
                                            (can be given multiple times)"))
                                .arg(Arg::with_name("dump-session-key")
                                     .long("dump-session-key")
                                     .help("Prints the session key to stderr")))

                    .subcommand(SubCommand::with_name("split")
                                .about("Splits a message into OpenPGP packets")
                                .arg(Arg::with_name("input").value_name("FILE")
                                     .help("Sets the input file to use"))
                                .arg(Arg::with_name("prefix").value_name("FILE")
                                     .long("prefix")
                                     .short("p")
                                     .help("Sets the prefix to use for output files \
                                            (defaults to the input filename with a dash, \
                                            or 'output')")))
                    .subcommand(SubCommand::with_name("join")
                                .about("Joins OpenPGP packets split across \
                                        files")
                                .arg(Arg::with_name("input").value_name("FILE")
                                     .multiple(true)
                                     .help("Sets the input files to use"))
                                .arg(Arg::with_name("output").value_name("FILE")
                                     .long("output")
                                     .short("o")
                                     .help("Sets the output file to use"))
                                .arg(Arg::with_name("kind")
                                     .value_name("KIND")
                                     .long("kind")
                                     .possible_values(&["message", "publickey",
                                                        "secretkey",
                                                        "signature", "file"])
                                     .default_value("file")
                                     .help("Selects the kind of header line to \
                                            produce"))
                                .arg(Arg::with_name("binary")
                                     .long("binary")
                                     .short("B")
                                     .help("Don't ASCII-armor encode the \
                                            OpenPGP data"))))

        .subcommand(SubCommand::with_name("wkd")
                    .about("Interacts with Web Key Directories")
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .subcommand(SubCommand::with_name("url")
                                .about("Prints the Web Key Directory URL of \
                                        an email address.")
                                .arg(Arg::with_name("input")
                                    .value_name("EMAIL_ADDRESS")
                                    .required(true)
                                    .help("The email address from which to \
                                            obtain the WKD URI.")))
                    .subcommand(SubCommand::with_name("get")
                                .about("Writes to the standard output the \
                                        Cert retrieved \
                                        from a Web Key Directory, given an \
                                        email address")
                                .arg(Arg::with_name("input")
                                    .value_name("EMAIL_ADDRESS")
                                    .required(true)
                                    .help("The email address from which to \
                                            obtain the Cert from a WKD."))
                                .arg(Arg::with_name("binary")
                                    .long("binary")
                                    .short("B")
                                    .help("Don't ASCII-armor encode the OpenPGP data")))
                    .subcommand(SubCommand::with_name("generate")
                                .about("Generates a Web Key Directory \
                                        for the given domain and keys.  \
                                        If the WKD exists, the new \
                                        keys will be inserted and it \
                                        is updated and existing ones \
                                        will be updated.")
                                .arg(Arg::with_name("base_directory")
                                     .value_name("WEB-ROOT")
                                     .required(true)
                                     .help("The location to write the WKD to. \
                                            This must be the directory the \
                                            webserver is serving the \
                                            '.well-known' directory from."))
                                .arg(Arg::with_name("domain")
                                    .value_name("DOMAIN")
                                    .help("The domain for the WKD.")
                                    .required(true))
                                .arg(Arg::with_name("input")
                                    .value_name("KEYRING")
                                    .help("The keyring file with the keys to add to the WKD."))
                                .arg(Arg::with_name("direct_method")
                                     .long("direct_method")
                                     .short("d")
                                     .help("Use the direct method. \
                                            [default: advanced method]"))
                    )
        )
}
