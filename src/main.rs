use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use futures::stream::StreamExt;
use reqwest::StatusCode;
use std::fs::OpenOptions;
use colored::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let učitane_poveznice_oglasa: Vec<String> = učitaj_poveznice_oglasa("oglasi.tekst")?;
    let mut trenutni_broj_oglasa:u32 = 0; let  ukupni_broj_oglasa:u32 = učitane_poveznice_oglasa.len();
    let iteratator_poveznica_oglasa = futures::stream::iter(
        učitane_poveznice_oglasa.into_iter().map(|poveznica_oglasa| {
        trenutni_broj_oglasa+=1;
        async move {
            let client = reqwest::Client::new();
            match client.get(&poveznica_oglasa)
            .header(reqwest::header::USER_AGENT, format!("🌱-{}",trenutni_broj_oglasa))
            .send()
            .await {
                Ok(resp) => {
                    match resp.status(){
                    StatusCode::OK => {
                        match resp.bytes().await {
                            Ok(slovnjaci) => {
                                let poruka_spremljeno = format!("Spremljeno {}/{} oglasa, primljeno {} slovnjaka sa poveznice {}",trenutni_broj_oglasa,ukupni_broj_oglasa,slovnjaci.len(), poveznica_oglasa);
                                eprintln!("{}",poruka_spremljeno.green().bold());
                                let mut datoteka_oglasa = OpenOptions::new()
                                .create(true)
                                .write(true)
                                .append(true)
                                .open(&poveznica_oglasa[39..75])
                                .unwrap();
                                let podatci: Result<Vec<_>, _> = slovnjaci.bytes().collect();
                                let podatci = podatci.expect("Zabuna prilikom učitavanja podataka!");
                                datoteka_oglasa.write_all(&podatci).expect("Zabuna prilikom upisivanja podataka!");
                            }
                            Err(zabuna) => eprintln!("Zabuna {} prilikom učitavanja slovnjaka, {}",zabuna, poveznica_oglasa),
                        }
                    },
                    _=>{
                        let poruka_neuspjeh = format!("Neuspjeh preuzimanja {}, {}/{} {}",resp.status(),trenutni_broj_oglasa,ukupni_broj_oglasa,poveznica_oglasa);
                        eprintln!("{}",poruka_neuspjeh.red().bold());
                        let mut datoteka_neuspjeh = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(format!("{}.tekst",resp.status().as_str()))
                        .unwrap();
                        writeln!(datoteka_neuspjeh,"{}",poveznica_oglasa).unwrap();
                    }
                    }
                }
                Err(zabuna) => {
                    let poruka_zabuna = format!("Zabuna {} prilikom preuzimanja, {}",zabuna,poveznica_oglasa);
                    eprintln!("{}",poruka_zabuna.yellow().bold());
                    let mut datoteka_zabuna = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open("zabuna.tekst")
                    .unwrap();
                    writeln!(datoteka_zabuna,"{}",poveznica_oglasa).unwrap();
                },
            }
        }
    })
    ).buffer_unordered(30).collect::<Vec<()>>();
    iteratator_poveznica_oglasa.await;

    Ok(())
}

fn učitaj_poveznice_oglasa(staza_do_datoteke_sa_oglasima: &str) -> std::io::Result<Vec<String>> {
    let datoteka_sa_oglasima = File::open(staza_do_datoteke_sa_oglasima)?;
    let čitač = BufReader::new(datoteka_sa_oglasima);
    Ok(
        čitač.lines().filter_map(Result::ok).collect()
    )
}
