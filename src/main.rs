use anyhow::Result;
use clap::Parser;
use console::{Alignment, Term};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// おかず
    side_dishes: Vec<String>,

    /// プレキャプション
    #[arg(short, long)]
    pre_captions: Vec<String>,

    /// アフターキャプション
    #[arg(short = 'A', long)]
    after_captions: Vec<String>,

    /// ファイルからスクリプトを読み込む
    #[arg(short = 'f', long,
        conflicts_with_all(["side_dishes", "pre_captions", "after_captions"]))
    ]
    script_file: Option<PathBuf>,

    #[command(flatten)]
    anime_args: Option<AnimeArgs>,
}

#[derive(clap::Args)]
struct AnimeArgs {
    /// アニメにするか
    #[arg(short, long)]
    anime: bool,

    /// アニメのインターバル
    #[arg(
        short, long,
        default_value_t = 1000,
        value_parser = clap::value_parser!(u64).range(10..))
    ]
    interval: u64,
}

#[derive(Deserialize, Debug)]
struct Script {
    side_dishes: Vec<String>,
    pre_captions: Vec<String>,
    after_captions: Vec<String>,
}

impl Script {
    fn load(path: &Path) -> Result<Self> {
        let script = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&script)?)
    }
}

fn main() -> Result<()> {
    let Args {
        side_dishes,
        pre_captions,
        after_captions,
        script_file,
        anime_args,
    } = Args::parse();

    let (side_dishes, pre_captions, after_captions) = match script_file {
        Some(path) => {
            let Script {
                side_dishes,
                pre_captions,
                after_captions,
            } = Script::load(&path)?;
            (side_dishes, pre_captions, after_captions)
        }
        None => (side_dishes, pre_captions, after_captions),
    };

    let term = Term::stdout();
    let terminal_width = term.size().1 as usize;

    let interval = match anime_args {
        Some(AnimeArgs {
            anime: true,
            interval,
        }) => {
            term.clear_screen()?;
            Some(interval)
        }
        _ => None,
    };

    let empty_dragon = create_dragon("", terminal_width);
    let mut printed_flag = false;

    let mut pre_captions_iter = pre_captions.into_iter().peekable();
    while let Some(pre_caption) = pre_captions_iter.next() {
        for line in empty_dragon.iter() {
            term.write_line(line)?;
        }
        let pre_caption = console::pad_str(&pre_caption, 60, Alignment::Center, None);
        term.write_line(&pre_caption)?;
        printed_flag = true;

        if let Some(interval) = interval {
            if pre_captions_iter.peek().is_some() {
                clear_dragon(interval, &term, &mut printed_flag)?;
            }
        }
    }

    let mut side_dish_iter = side_dishes.iter().peekable();

    if let Some(interval) = interval {
        if printed_flag && side_dish_iter.peek().is_some() {
            clear_dragon(interval, &term, &mut printed_flag)?;
        }
    }

    while let Some(side_dish) = side_dish_iter.next() {
        let dragon = create_dragon(side_dish, terminal_width);
        for line in dragon {
            term.write_line(&line)?;
        }
        let empty_line = console::pad_str("", 60, Alignment::Center, None);
        term.write_line(&empty_line)?;
        printed_flag = true;

        if let Some(interval) = interval {
            if side_dish_iter.peek().is_some() {
                clear_dragon(interval, &term, &mut printed_flag)?;
            }
        }
    }

    let mut after_captions_iter = after_captions.into_iter().peekable();

    if let Some(interval) = interval {
        if printed_flag && after_captions_iter.peek().is_some() {
            clear_dragon(interval, &term, &mut printed_flag)?;
        }
    }

    while let Some(after_caption) = after_captions_iter.next() {
        for line in empty_dragon.iter() {
            term.write_line(line)?;
        }
        let after_caption = console::pad_str(&after_caption, 60, Alignment::Center, None);
        term.write_line(&after_caption)?;

        if let Some(interval) = interval {
            if after_captions_iter.peek().is_some() {
                clear_dragon(interval, &term, &mut printed_flag)?;
            }
        }
    }

    Ok(())
}

fn clear_dragon(interval: u64, term: &Term, printed_flag: &mut bool) -> Result<()> {
    sleep(Duration::from_millis(interval));
    term.clear_screen()?;
    *printed_flag = false;

    Ok(())
}

fn create_dragon(side_dish: &str, terminal_width: usize) -> Vec<String> {
    let side_dish: Vec<char> = side_dish.chars().collect();
    let empty = Vec::new();
    let lines: Vec<String> = match side_dish.len() {
        0..=8 => vec![side_dish.as_slice(), empty.as_slice()],
        9..=16 => vec![&side_dish[..8], &side_dish[8..]],
        _ => vec![&side_dish[..8], &side_dish[8..16]],
    }
    .into_iter()
    .map(|s| {
        let s = s.iter().collect::<String>();
        console::pad_str(&s, 20, Alignment::Center, None).to_string()
    })
    .collect();

    #[rustfmt::skip]
    let dragon = "                                          ,. ､
                                        く  r',ゝ
r'￣￣￣￣￣￣￣￣￣ヽ                   ,ゝｰ'､
|                    |          ､      ／      ヽ.
|                    |        く、｀ヽ/  ∩       |
|$line1$ ＞        ｀＞             |
|$line2$|         く´ , -'7         レ个ー─┐
|                    |          ｀´   //  /      ー个ー─'7
|                    |               //  /         |    (
ゝ＿＿＿＿＿＿＿＿__ノ              //  /'┤      |ヽv'⌒ヽ､ゝ
                                   くﾉ  lｰ┤       ヽ.
                                    ｀^^'ｰ┤          ▽_
                                    ((    )          ヽ乙_
                                    ((    )ヽ､          ヽレl
                                    ≧＿_ゝ    ｀ﾞー-=､.＿_,ゝ";

    dragon
        .replace("$line1$", &lines[0])
        .replace("$line2$", &lines[1])
        .lines()
        .map(|line| console::pad_str(line, terminal_width, Alignment::Left, None).to_string())
        .collect()
}
