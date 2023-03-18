mod queryable;
mod value_borrow;
mod value_intern;

use anyhow::Result;
use bstr::ByteSlice;
use memmap::MmapOptions;
use queryable::Queryable;
use rayon::prelude::*;
use serde_json::from_slice;
use std::{fs::File, path::PathBuf, str::FromStr, time::Instant};
use structopt::StructOpt;
use value_borrow::ValueBorrow;
use value_intern::ValueIntern;

#[derive(Debug)]
enum ValueType {
    Serde,
    Borrow,
    Intern,
}

impl FromStr for ValueType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "serde" => Ok(Self::Serde),
            "borrow" => Ok(Self::Borrow),
            "intern" => Ok(Self::Intern),
            _ => Err(anyhow::anyhow!("unexpected value type '{s}'")),
        }
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long)]
    input: PathBuf,

    #[structopt(long)]
    value_type: ValueType,
}

fn pred(v: &impl Queryable) -> bool {
    v.get_all("subArts")
        .into_iter()
        .flat_map(|v| v.get_all("subSubArts"))
        .flat_map(|v| v.get_all("size"))
        .any(|v| v.contains("snug"))
}

fn main() -> Result<()> {
    let start = Instant::now();
    let opt = Opt::from_args();
    let f = File::open(opt.input)?;
    let mmap = unsafe { MmapOptions::new().map(&f)? };
    let bytes: &[u8] = mmap.as_ref();
    let n = match opt.value_type {
        ValueType::Serde => {
            println!("Using serde_json::Value");
            bytes
                .lines()
                .par_bridge()
                .flat_map(from_slice::<serde_json::Value>)
                .filter(pred)
                .count()
        }
        ValueType::Borrow => {
            println!("Using ValueBorrow");
            bytes
                .lines()
                .par_bridge()
                .flat_map(from_slice::<ValueBorrow>)
                .filter(pred)
                .count()
        }
        ValueType::Intern => {
            println!("Using ValueIntern");
            bytes
                .lines()
                .par_bridge()
                .flat_map(from_slice::<ValueIntern>)
                .filter(pred)
                .count()
        }
    };
    dbg!(n);
    dbg!(start.elapsed());

    Ok(())
}
