use csv::ReaderBuilder;
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::path::PathBuf;

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();

    // Если CSV перетащили на программу — используем его.
    // Иначе ищем стандартный CSV рядом с EXE.
    let file_path = if let Some(path) = args.get(1) {
        PathBuf::from(path)
    } else {
        let exe = std::env::current_exe()?;
        let dir = exe.parent().unwrap_or(std::path::Path::new("."));
        dir.join("funpay_sales_to20261014.csv")
    };

    println!("Файл: {}", file_path.display());
    println!("Чтение данных...\n");

    let mut reader = ReaderBuilder::new().flexible(true).from_path(&file_path)?;

    let headers = reader.headers()?.clone();

    let find_column = |name: &str| {
        headers
            .iter()
            .position(|h| h.trim_start_matches('\u{feff}') == name)
            .ok_or_else(|| format!("Не найден столбец {name}"))
    };

    let game_idx = find_column("game_name")?;
    let currency_idx = find_column("currency")?;
    let amount_idx = find_column("amount")?;

    // Суммируем в копейках, чтобы избежать ошибок округления.
    let mut totals: HashMap<String, i64> = HashMap::new();

    let mut total_rows = 0_u64;
    let mut rub_rows = 0_u64;
    let mut skipped_rows = 0_u64;

    for result in reader.records() {
        total_rows += 1;

        let record = match result {
            Ok(record) => record,
            Err(err) => {
                eprintln!("Ошибка чтения строки {total_rows}: {err}");
                skipped_rows += 1;
                continue;
            }
        };

        if record.get(currency_idx).unwrap_or("").trim() != "RUB" {
            continue;
        }

        let game = record.get(game_idx).unwrap_or("").trim();
        let amount_str = record.get(amount_idx).unwrap_or("").trim();

        if game.is_empty() {
            skipped_rows += 1;
            continue;
        }

        let amount = match amount_str.parse::<f64>() {
            Ok(value) if value.is_finite() => value,
            _ => {
                eprintln!("Пропуск строки {total_rows}: неверная сумма '{amount_str}'");
                skipped_rows += 1;
                continue;
            }
        };

        let kopecks = (amount * 100.0).round() as i64;

        *totals.entry(game.to_string()).or_insert(0) += kopecks;
        rub_rows += 1;
    }

    let mut games: Vec<_> = totals.into_iter().collect();
    games.sort_by(|a, b| b.1.cmp(&a.1));

    println!("==============================================================");
    println!("                  FUNPAY SALES ANALYZER");
    println!("==============================================================");
    println!("Всего строк:           {total_rows}");
    println!("Учтено операций в RUB: {rub_rows}");
    println!("Пропущено строк:       {skipped_rows}");
    println!("--------------------------------------------------------------");

    println!("{:<42} {:>18}", "ИГРА", "СУММА (RUB)");
    println!("{}", "-".repeat(62));

    let mut grand_total: i64 = 0;

    for (game, kopecks) in &games {
        grand_total += *kopecks;
        println!(
            "{:<42} {:>18}.{:02}",
            truncate(&game, 42),
            kopecks / 100,
            kopecks.abs() % 100
        );
    }

    println!("{}", "-".repeat(62));
    println!(
        "{:<42} {:>18}.{:02}",
        "ИТОГО",
        grand_total / 100,
        grand_total.abs() % 100
    );
    println!("Количество игр: {}", games.len());

    Ok(())
}

fn truncate(text: &str, max_chars: usize) -> String {
    let mut chars = text.chars();
    let result: String = chars.by_ref().take(max_chars).collect();

    if chars.next().is_some() {
        result
    } else {
        text.to_string()
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("\nОШИБКА: {err}");
    }

    println!("\nНажми Enter, чтобы закрыть окно...");
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}
