use crate::helper::spawn_app;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use reqwest::{Client, multipart};

#[tokio::test]
async fn post_shows_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = Client::new();

    let d = NaiveDate::from_ymd_opt(2025, 4, 14).unwrap();
    let t = NaiveTime::from_hms_opt(23, 17, 0).unwrap();
    let dt = NaiveDateTime::new(d, t);
    let date = dt.format("%Y-%m-%dT%H:%M:%S").to_string();
    let venue = "The Jungle";

    let file_bytes = tokio::fs::read("tests/test_poster.pdf")
        .await
        .expect("Failed to read test file");

    let test_file = multipart::Part::bytes(file_bytes)
        .file_name("test_poster.pdf")
        .mime_str("application/pdf")
        .unwrap();

    let form = multipart::Form::new()
        .text("date", date)
        .part("poster", test_file)
        .text("venue", venue);

    let response = client
        .post(&format!("{}/shows", &app.address))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", form.boundary()),
        )
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT date, venue FROM shows",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved show.");

    assert_eq!(saved.date, dt);
    assert_eq!(saved.venue, venue);
}

#[tokio::test]
async fn post_shows_returns_400_for_bad_poster_file() {
    let app = spawn_app().await;
    let client = Client::new();

    let d = NaiveDate::from_ymd_opt(2025, 4, 14).unwrap();
    let t = NaiveTime::from_hms_opt(23, 17, 0).unwrap();
    let dt = NaiveDateTime::new(d, t);
    let date = dt.format("%Y-%m-%dT%H:%M:%S").to_string();
    let venue = "The Jungle";

    let file_bytes = tokio::fs::read("tests/bad_file.txt")
        .await
        .expect("Failed to read test file");

    let test_file = multipart::Part::bytes(file_bytes)
        .file_name("bad_file.txt")
        .mime_str("text/plain")
        .unwrap();

    let form = multipart::Form::new()
        .text("date", date)
        .part("poster", test_file)
        .text("venue", venue);

    let response = client
        .post(&format!("{}/shows", &app.address))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", form.boundary()),
        )
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn post_shows_excluding_optional_poster_file_returns_200() {
    let app = spawn_app().await;
    let client = Client::new();

    let form = multipart::Form::new()
        .text("date", "2024-04-14T16:42:00")
        .text("venue", "The Jungle");

    let response = client
        .post(&format!("{}/shows", &app.address))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", form.boundary()),
        )
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn post_shows_with_missing_data_returns_400() {
    let app = spawn_app().await;
    let client = Client::new();

    // no date
    let form = multipart::Form::new().text("venue", "The Jungle");

    let response = client
        .post(&format!("{}/shows", &app.address))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", form.boundary()),
        )
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());

    // no venue
    let form = multipart::Form::new().text("date", "2024-04-14T16:42:00");

    let response = client
        .post(&format!("{}/shows", &app.address))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", form.boundary()),
        )
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());

    // date bad format
    let form = multipart::Form::new()
        .text("venue", "The Jungle")
        .text("date", "2024-04-14");

    let response = client
        .post(&format!("{}/shows", &app.address))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", form.boundary()),
        )
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());
}
