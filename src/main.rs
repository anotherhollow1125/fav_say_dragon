use anyhow::Result;
use clap::{Parser, Subcommand};
use console::{Alignment, Term};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

#[derive(Parser)]
#[command(version, about, flatten_help = true)]
struct Args {
    #[command(subcommand)]
    sub: Command,
}

#[derive(Subcommand, Clone, Debug)]
enum Command {
    /// 一度に出力
    Say {
        /// おかず
        side_dish: String,
        /// キャプション
        caption: Option<String>,
    },
    /// アニメーション出力
    Anime {
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

        /// インターバル (ms)
        #[arg(
            short, long,
            default_value_t = 1000,
            value_parser = clap::value_parser!(u64).range(10..))
        ]
        interval: u64,
    },
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
    match Args::parse().sub {
        Command::Say { side_dish, caption } => say(&side_dish, caption.as_deref())?,
        Command::Anime {
            side_dishes,
            pre_captions,
            after_captions,
            script_file,
            interval,
        } => anime(
            side_dishes,
            pre_captions,
            after_captions,
            script_file,
            interval,
        )?,
    }

    Ok(())
}

fn say(side_dish: &str, caption: Option<&str>) -> Result<()> {
    let term = Term::stdout();
    let terminal_width = term.size().1 as usize;

    let dragon = create_dragon(side_dish, terminal_width);
    for line in dragon {
        term.write_line(&line)?;
    }
    let caption = console::pad_str(caption.unwrap_or(""), 60, Alignment::Center, None);
    term.write_line(&caption)?;

    Ok(())
}

fn anime(
    side_dishes: Vec<String>,
    pre_captions: Vec<String>,
    after_captions: Vec<String>,
    script_file: Option<PathBuf>,
    interval: u64,
) -> Result<()> {
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
    term.clear_screen()?;
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

        if pre_captions_iter.peek().is_some() {
            clear_dragon(interval, &term, &mut printed_flag)?;
        }
    }

    let mut side_dish_iter = side_dishes.iter().peekable();

    if printed_flag && side_dish_iter.peek().is_some() {
        clear_dragon(interval, &term, &mut printed_flag)?;
    }

    while let Some(side_dish) = side_dish_iter.next() {
        let dragon = create_dragon(side_dish, terminal_width);
        for line in dragon {
            term.write_line(&line)?;
        }
        let empty_line = console::pad_str("", 60, Alignment::Center, None);
        term.write_line(&empty_line)?;
        printed_flag = true;

        if side_dish_iter.peek().is_some() {
            clear_dragon(interval, &term, &mut printed_flag)?;
        }
    }

    let mut after_captions_iter = after_captions.into_iter().peekable();

    if printed_flag && after_captions_iter.peek().is_some() {
        clear_dragon(interval, &term, &mut printed_flag)?;
    }

    while let Some(after_caption) = after_captions_iter.next() {
        for line in empty_dragon.iter() {
            term.write_line(line)?;
        }
        let after_caption = console::pad_str(&after_caption, 60, Alignment::Center, None);
        term.write_line(&after_caption)?;

        if after_captions_iter.peek().is_some() {
            clear_dragon(interval, &term, &mut printed_flag)?;
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
    let lines: Vec<String> = match side_dish.lines().count() {
        0 => vec!["".to_string(), "".to_string()],
        1 => {
            let empty = Vec::new();
            let side_dish: Vec<char> = side_dish.chars().collect();
            match side_dish.len() {
                0..=16 => vec![side_dish.as_slice(), empty.as_slice()],
                17..=32 => vec![&side_dish[..16], &side_dish[16..]],
                _ => vec![&side_dish[..16], &side_dish[16..32]],
            }
            .into_iter()
            .map(|s| s.iter().collect::<String>())
            .collect()
        }
        _ => {
            let mut lines: Vec<String> = side_dish.lines().rev().map(|s| s.to_string()).collect();
            let line0 = lines.pop().unwrap_or("".to_string());
            let line1 = lines.pop().unwrap_or("".to_string());
            vec![line0, line1]
        }
    }
    .into_iter()
    .map(|s| console::pad_str(&s, 20, Alignment::Center, None).to_string())
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
