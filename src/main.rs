use csv::ReaderBuilder;
use std::collections::HashMap;
use std::error::Error;

const FILE_NAME: &str = "funpay_sales_to20261014.csv";

fn main() -> Result<(), Box<dyn Error>> {
    let mut reader = ReaderBuilder::new().flexible(true).from_path(FILE_NAME)?;

    let headers = reader.headers()?.clone();

    let game_idx = headers
        .iter()
        .position(|h| h == "game_name")
        .ok_or("Не найден столбец game_name")?;

    let currency_idx = headers
        .iter()
        .position(|h| h == "currency")
        .ok_or("Не найден столбец currency")?;

    let amount_idx = headers
        .iter()
        .position(|h| h == "amount")
        .ok_or("Не найден столбец amount")?;

    // Храним суммы в копейках, чтобы избежать ошибок округления.
    let mut totals: HashMap<String, i64> = HashMap::new();

    let mut total_rows = 0_u64;
    let mut rub_rows = 0_u64;
    let mut skipped_rows = 0_u64;

    for result in reader.records() {
        total_rows += 1;

        let record = match result {
            Ok(record) => record,
            Err(err) => {
                eprintln!("Ошибка чтения строки: {err}");
                skipped_rows += 1;
                continue;
            }
        };

        let currency = record.get(currency_idx).unwrap_or("").trim();

        // Учитываем только рублёвые операции.
        if currency != "RUB" {
            continue;
        }

        let game_name = record.get(game_idx).unwrap_or("").trim();

        let amount_str = record.get(amount_idx).unwrap_or("").trim();

        if game_name.is_empty() {
            eprintln!("Пропущена строка {total_rows}: пустое название игры");
            skipped_rows += 1;
            continue;
        }

        let amount: f64 = match amount_str.parse::<f64>() {
            Ok(value) if value.is_finite() => value,
            _ => {
                eprintln!("Пропущена строка {total_rows}: неверная сумма '{amount_str}'");
                skipped_rows += 1;
                continue;
            }
        };

        let kopecks = (amount * 100.0).round() as i64;

        *totals.entry(game_name.to_string()).or_insert(0) += kopecks;
        rub_rows += 1;
    }

    // Сортируем игры по сумме продаж: от большей к меньшей.
    let mut games: Vec<_> = totals.into_iter().collect();
    games.sort_by(|a, b| b.1.cmp(&a.1));

    println!();
    println!("==============================================");
    println!("       СТАТИСТИКА ПРОДАЖ FUNPAY");
    println!("==============================================");
    println!("Файл: {FILE_NAME}");
    println!("Всего обработано строк: {total_rows}");
    println!("Учтено рублёвых строк:  {rub_rows}");
    println!("Пропущено строк:        {skipped_rows}");
    println!("==============================================");
    println!();

    println!("{:<50} {:>18}", "ИГРА", "СУММА (RUB)");
    println!("{}", "-".repeat(70));

    let mut grand_total: i64 = 0;

    for (game, kopecks) in &games {
        let rubles = *kopecks as f64 / 100.0;
        grand_total += kopecks;

        println!("{:<50} {:>18.2}", game, rubles);
    }

    println!("{}", "-".repeat(70));
    println!("{:<50} {:>18.2}", "ИТОГО", grand_total as f64 / 100.0);
    println!();
    println!("Количество игр: {}", games.len());

    Ok(())
}
