import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import BatteryMonitorComponent from "./components/BatteryMonitorComponent";

const SystemInfoComponent: React.FC = () => {
  const [systemInfo, setSystemInfo] = useState<string>("Loading...");

  const fetchSystemInfo = async () => {
    try {
      const info = await invoke("get_full_system_info"); // Call the Rust command
      setSystemInfo(info as string); // Set the retrieved info to state
    } catch (error) {
      console.error("Error fetching system info:", error);
      setSystemInfo("Failed to retrieve system information.");
    }
  };

  useEffect(() => {
    fetchSystemInfo(); // Fetch system info on component mount
  }, []);

  return (
    <div>
      <h2>System Information</h2>
      <p>{systemInfo}</p>
      <BatteryMonitorComponent />
    </div>
  );
};

export default SystemInfoComponent;
