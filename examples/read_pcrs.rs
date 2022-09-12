use tpm2::{
    commands::PcrRead,
    os::get_default_tpm,
    types::{tpm, tpml, tpms},
    Command,
};

fn main() {
    let mut tpm = get_default_tpm().expect("Unable to open TPM");

    // TODO: Query the available banks and read _all_ the PCRs.

    let sel = [
        tpms::PcrSelection {
            hash: tpm::Alg::SHA1,
            select: [true; tpms::NUM_PCRS],
        },
        tpms::PcrSelection {
            hash: tpm::Alg::SHA256,
            select: [true; tpms::NUM_PCRS],
        },
    ];

    let cmd = PcrRead {
        pcr_selection: tpml::PcrSelection::from(&sel),
    };
    println!("Reading from {:?} PCR banks", cmd.pcr_selection.len());
    let rsp = cmd.run(&mut tpm).expect("Unable to read PCRS");

    for sel in rsp.pcr_selection {
        print!("{:?} selection:", sel.hash);
        for (i, &bit) in sel.select.iter().enumerate() {
            if bit {
                print!(" {}", i);
            }
        }
        println!();
    }

    println!("{} digests", rsp.pcr_values.len());
    for digest in rsp.pcr_values {
        print!("\t0x");
        for &b in digest {
            print!("{:02X}", b);
        }
        println!()
    }
}
