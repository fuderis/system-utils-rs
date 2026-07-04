#![cfg(feature = "media")]
use system_utils::MediaControl;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("== Media Control Example ==\n");

    let metadata = MediaControl::metadata().await?;
    println!("Metadata:");
    println!("  Title    : {}", metadata.title);
    println!("  Artist   : {}", metadata.artist);
    println!("  Album    : {}", metadata.album);
    println!("  Duration : {:?}", metadata.duration);
    println!("  Position : {:?}", metadata.position);
    println!("  Playing  : {}", metadata.playing);
    println!("  Artwork  : {:?}", metadata.artwork_url);

    println!();

    let position = MediaControl::position().await?;
    println!("Current position: {:?}", position);

    let duration = MediaControl::duration().await?;
    println!("Duration: {:?}", duration);

    println!("\nPausing playback...");
    MediaControl::pause().await?;
    sleep(Duration::from_secs(2)).await;

    println!("Resuming playback...");
    MediaControl::play().await?;
    sleep(Duration::from_secs(2)).await;

    println!("Toggling play/pause...");
    MediaControl::play_pause().await?;
    sleep(Duration::from_secs(2)).await;

    println!("Toggling play/pause again...");
    MediaControl::play_pause().await?;
    sleep(Duration::from_secs(2)).await;

    println!("Seeking forward 10 seconds...");
    MediaControl::seek_forward(10).await?;
    sleep(Duration::from_secs(2)).await;

    println!("Seeking backward 5 seconds...");
    MediaControl::seek_backward(5).await?;
    sleep(Duration::from_secs(2)).await;

    println!("Next track...");
    MediaControl::next_track().await?;
    sleep(Duration::from_secs(3)).await;

    println!("Previous track...");
    MediaControl::previous_track().await?;
    sleep(Duration::from_secs(3)).await;

    println!("Stopping playback...");
    MediaControl::stop().await?;

    println!("\nDone.");

    Ok(())
}
