import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import './App.css';

function App() {
  // State management
  const [isConnected, setIsConnected] = useState(false);
  const [isMonitoring, setIsMonitoring] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState('Checking League Client...');
  const [monitoringStatus, setMonitoringStatus] = useState('Disabled');
  const [isConnecting, setIsConnecting] = useState(false);
  const [isToggling, setIsToggling] = useState(false);
  const [isManualAccepting, setIsManualAccepting] = useState(false);
  const [logs, setLogs] = useState([
    { time: new Date(), message: 'Application started. Click "Connect to League" to begin.', type: 'info' }
  ]);

  const addLogEntry = (message: string, type: string = 'info') => {
    const newLog = {
      time: new Date(),
      message,
      type
    };
    setLogs(prevLogs => {
      const newLogs = [...prevLogs, newLog];
      return newLogs.slice(-50); 
    });
  };

  useEffect(() => {
    const setupEventListeners = async () => {
      try {
        const unlistenMatchAccepted = await listen('match-accepted', (event) => {
          addLogEntry(`🎉 ${event.payload}`, 'success');
        });

        const unlistenMatchFailed = await listen('match-accept-failed', (event) => {
          addLogEntry(`❌ ${event.payload}`, 'error');
        });

        const unlistenDisconnected = await listen('league-disconnected', (event) => {
          setIsConnected(false);
          setConnectionStatus('League Client disconnected');
          addLogEntry(`⚠️ ${event.payload}`, 'error');
        });

        const unlistenAppReady = await listen('app-ready', () => {
          addLogEntry('Application ready. Checking for League Client...', 'info');
          connectToLeague();
        });

        return () => {
          unlistenMatchAccepted();
          unlistenMatchFailed();
          unlistenDisconnected();
          unlistenAppReady();
        };
      } catch (error) {
        console.error('Failed to setup event listeners:', error);
      }
    };

    setupEventListeners();
    checkAutoAcceptStatus();
  }, []);

  const connectToLeague = async () => {
    try {
      setIsConnecting(true);
      const result = await invoke<string>('connect_to_league');
      
      setIsConnected(true);
      setConnectionStatus(result);
      addLogEntry(result, 'success');
      
    } catch (error) {
      setIsConnected(false);
      setConnectionStatus('Connection failed');
      addLogEntry(`Connection failed: ${error}`, 'error');
    } finally {
      setIsConnecting(false);
    }
  };

  const toggleAutoAccept = async () => {
    try {
      setIsToggling(true);
      
      if (isMonitoring) {
        const result = await invoke<string>('stop_auto_accept');
        setIsMonitoring(false);
        setMonitoringStatus('Disabled');
        addLogEntry(result, 'info');
      } else {
        const result = await invoke<string>('start_auto_accept');
        setIsMonitoring(true);
        setMonitoringStatus('Monitoring for matches...');
        addLogEntry(result, 'success');
      }
    } catch (error) {
      addLogEntry(`Auto-accept toggle failed: ${error}`, 'error');
    } finally {
      setIsToggling(false);
    }
  };

  const manualAccept = async () => {
    try {
      setIsManualAccepting(true);
      const result = await invoke<string>('manual_accept');
      addLogEntry(result, 'success');
    } catch (error) {
      addLogEntry(`Manual accept failed: ${error}`, 'error');
    } finally {
      setIsManualAccepting(false);
    }
  };

  const checkAutoAcceptStatus = async () => {
    try {
      const isRunning = await invoke<boolean>('is_auto_accept_running');
      if (isRunning) {
        setIsMonitoring(true);
        setMonitoringStatus('Monitoring for matches...');
      }
    } catch (error) {
      console.error('Failed to check auto-accept status:', error);
    }
  };


  return (
    <div className="app">
      <div className="container">
        
        <div className="header">
          <h1>Lolytics Auto Accept</h1>
          <p>Never miss a match again</p>
        </div>

        <div className="status-card">
          <div className="status-indicator">
            <div className={`status-dot ${isConnected ? 'connected' : ''}`}></div>
            <strong>Connection Status</strong>
          </div>
          <div className="status-text">{connectionStatus}</div>
        </div>

        <div className="status-card">
          <div className="status-indicator">
            <div className={`status-dot ${isMonitoring ? 'monitoring' : ''}`}></div>
            <strong>Auto-Accept Status</strong>
          </div>
          <div className="status-text">{monitoringStatus}</div>
        </div>

        <div className="button-group">
          <button 
            className="btn btn-secondary" 
            onClick={connectToLeague}
            disabled={isConnecting}
          >
            {isConnecting ? 'Connecting...' : isConnected ? 'Connected ✓' : 'Connect to League'}
          </button>
          
          <button 
            className={`btn ${isMonitoring ? 'btn-danger' : 'btn-primary'}`}
            onClick={toggleAutoAccept}
            disabled={!isConnected || isToggling}
          >
            {isToggling 
              ? 'Starting...' 
              : isMonitoring 
                ? 'Disable Auto-Accept' 
                : 'Enable Auto-Accept'
            }
          </button>
          
          <button 
            className="btn btn-secondary"
            onClick={manualAccept}
            disabled={!isConnected || isManualAccepting}
          >
            {isManualAccepting ? 'Accepting...' : 'Manual Accept'}
          </button>
        </div>

        <div className="log-container">
          <div className="log-content">
            {logs.map((log, index) => (
              <div key={index} className={`log-entry ${log.type}`}>
                [{log.time.toLocaleTimeString()}] {log.message}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;