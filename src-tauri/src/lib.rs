use battery::{Manager, State}; // Import the battery crate
use notify_rust::Notification; // For notifications
use rodio::{
    source::{Buffered, Source},
    Decoder, OutputStream,
}; // For sound playback
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::process::Command; // Import Command for executing system commands
use std::sync::{Arc, Mutex}; // For thread-safe shared state
use std::thread;
use std::time::Duration;
use tauri::{command, AppHandle};
use tauri_plugin_notification;

#[command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[command]
fn get_full_system_info() -> Result<String, String> {
    // Retrieve OS type and release using sys_info crate
    let os_type = sys_info::os_type().map_err(|e| e.to_string())?;
    let os_release = sys_info::os_release().map_err(|e| e.to_string())?;

    // Get additional device info based on OS
    let model = if cfg!(target_os = "macos") {
        // Get macOS model using sysctl
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("hw.model")
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            "Unknown Model".to_string()
        }
    } else {
        "Generic Device".to_string() // Placeholder for other OS types
    };

    // Format the device info without CPU details
    let device_info = format!("{} - {}", os_type, os_release);

    Ok(format!("{} - Model: {}", device_info, model))
}

#[command]
fn start_battery_monitor(app: AppHandle) {
    thread::spawn(move || {
        let manager = Manager::new().unwrap();
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();

        // Load the sound file into a buffered source
        let sound_file = BufReader::new(
            File::open("/usr/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga").unwrap(),
        );
        let source = Decoder::new(sound_file).unwrap().buffered(); // Use buffered source

        // Initialize shared state for alert playback status and last alert state
        let is_alert_playing = Arc::new(Mutex::new(false));
        let last_alert_state = Arc::new(Mutex::new(None)); // Track last alert state

        loop {
            for maybe_battery in manager.batteries().unwrap() {
                let battery = maybe_battery.unwrap();

                // Get the current state of charge and state
                let charge = battery
                    .state_of_charge()
                    .get::<battery::units::ratio::percent>();
                let state = battery.state();

                // Determine current alert condition
                let current_alert_condition = match (state, charge) {
                    (State::Charging, charge) if charge > 70.0 => Some("charging"),
                    (State::Discharging, charge) if charge < 31.0 => Some("discharging"),
                    _ => None,
                };

                let mut last_state_guard = last_alert_state.lock().unwrap();

                // Check if we need to send an alert based on changes in state
                if current_alert_condition != *last_state_guard {
                    *last_state_guard = current_alert_condition;

                    match current_alert_condition {
                        Some("charging") => {
                            let message = format!(
                                "⚠️ Unplug your laptop! Battery is at {:.0}% while charging.",
                                charge
                            );
                            println!("{}", message);
                            play_alert(&stream_handle, &source, &is_alert_playing).unwrap();
                            send_notification("Battery Alert", &message).unwrap();
                        }
                        Some("discharging") => {
                            let message = format!(
                                "⚠️ Plug in your laptop! Battery is at {:.0}% and discharging.",
                                charge
                            );
                            println!("{}", message);
                            play_alert(&stream_handle, &source, &is_alert_playing).unwrap();
                            send_notification("Battery Alert", &message).unwrap();
                        }
                        _ => {}
                    }
                }
            }

            // Wait for 20 seconds before checking again
            thread::sleep(Duration::from_secs(20));
        }
    });
}

fn play_alert(
    stream_handle: &rodio::OutputStreamHandle,
    source: &Buffered<Decoder<BufReader<File>>>,
    is_alert_playing: &Arc<Mutex<bool>>,
) -> Result<(), Box<dyn Error>> {
    let mut playing = is_alert_playing.lock().unwrap();

    if *playing {
        return Ok(()); // If already playing, do nothing.
    }

    *playing = true; // Set alert as playing

    match stream_handle.play_raw(source.clone().convert_samples()) {
        Ok(_) => {
            thread::sleep(Duration::from_secs(3)); // Play for a fixed duration (adjust as necessary)
            *playing = false; // Reset after playback duration
            Ok(())
        }
        Err(e) => {
            eprintln!("Error playing alert sound: {}", e);
            send_notification("Error", "Failed to play alert sound.").unwrap();
            *playing = false; // Reset on error
            Err(Box::new(e))
        }
    }
}

fn send_notification(title: &str, message: &str) -> Result<(), Box<dyn Error>> {
    Notification::new().summary(title).body(message).show()?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init()) // Register the notification plugin
        .invoke_handler(tauri::generate_handler![
            greet,
            get_full_system_info,
            start_battery_monitor,
        ]) // Register commands
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
