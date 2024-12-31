import React, { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

const BatteryStatusComponent: React.FC = () => {
  useEffect(() => {
    const startMonitoring = async () => {
      try {
        // Check if permission is granted
        let permissionGranted = await isPermissionGranted();

        // Request permission if not granted
        if (!permissionGranted) {
          const permission = await requestPermission();
          permissionGranted = permission === "granted";
        }

        // Start battery monitoring if permission is granted
        if (permissionGranted) {
          await invoke("start_battery_monitor"); // Start monitoring when component mounts
          console.log("Battery monitoring started.");
        } else {
          console.warn("Notification permission not granted.");
        }
      } catch (error) {
        console.error("Error starting battery monitor:", error);
      }
    };

    startMonitoring(); // Call function to start monitoring on mount
  }, []);

  const handleSendNotification = async () => {
    const permissionGranted = await isPermissionGranted();
    if (permissionGranted) {
      sendNotification({
        title: "Battery Monitor",
        body: "Monitoring started!",
      });
    } else {
      console.warn("Cannot send notification: Permission not granted.");
    }
  };

  return (
    <div>
      <h2>Battery Monitoring</h2>
      <p>Monitoring started... Check your notifications!</p>
      <button onClick={handleSendNotification}>Send Test Notification</button>
    </div>
  );
};

export default BatteryStatusComponent;
