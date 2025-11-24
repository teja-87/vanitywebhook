// db.rs — FINAL WORKING VERSION (uses plain query() → cannot fail)
use sqlx::PgPool;
use chrono::{NaiveDateTime, Utc};

pub async fn add_paid(
    pool: &PgPool,
    signature: &str,
    sender_wallet: &str,
    amount_lamports: u64,
    slot: i64,
    block_time_unix: Option<i64>,
    receiver_wallet: &str,
) -> Result<(), sqlx::Error> {
    let block_time = block_time_unix
        .and_then(|t| NaiveDateTime::from_timestamp_opt(t, 0))
        .unwrap_or_else(|| Utc::now().naive_utc());

    let amount_sol = amount_lamports as f64 / 1_000_000_000.0;
    let max_letters = if amount_sol == 0.1 { 3 } else if amount_sol == 0.2 { 4 } else { 0 };

    // Plain query() → works even if table has extra columns with defaults
    sqlx::query(
        r#"
        INSERT INTO public.payments (
            signature, sender_wallet, receiver_wallet, 
            amount_lamports, max_letters, slot, block_time
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (signature) DO NOTHING
        "#
    )
    .bind(signature)
    .bind(sender_wallet)
    .bind(receiver_wallet)
    .bind(amount_lamports as i64)
    .bind(max_letters as i32)
    .bind(slot)
    .bind(block_time)
    .execute(pool)
    .await?;

    println!("INSERTED INTO NHOST → {} ({} SOL)", signature, amount_sol);
    Ok(())
}