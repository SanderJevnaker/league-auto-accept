import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import './App.css';

interface ChampSelectConfig {
  auto_pick_enabled: boolean;
  auto_ban_enabled: boolean;
  pick_priority: string[];
  ban_priority: string[];
}

function App() {
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

  const [showSettings, setShowSettings] = useState(false);
  const [availableChampions, setAvailableChampions] = useState<string[]>([]);
  const [config, setConfig] = useState<ChampSelectConfig>({
    auto_pick_enabled: false,
    auto_ban_enabled: false,
    pick_priority: ['Jinx', 'Ashe', 'Caitlyn'],
    ban_priority: ['Yasuo', 'Zed', 'Master Yi']
  });

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

        const unlistenChampionPicked = await listen('champion-picked', (event) => {
          addLogEntry(`🎯 ${event.payload}`, 'success');
        });

        const unlistenChampionBanned = await listen('champion-banned', (event) => {
          addLogEntry(`🚫 ${event.payload}`, 'success');
        });

        const unlistenPickFailed = await listen('champion-pick-failed', (event) => {
          addLogEntry(`⚠️ ${event.payload}`, 'error');
        });

        const unlistenBanFailed = await listen('champion-ban-failed', (event) => {
          addLogEntry(`⚠️ ${event.payload}`, 'error');
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
          unlistenChampionPicked();
          unlistenChampionBanned();
          unlistenPickFailed();
          unlistenBanFailed();
          unlistenDisconnected();
          unlistenAppReady();
        };
      } catch (error) {
        console.error('Failed to setup event listeners:', error);
      }
    };

    setupEventListeners();
    checkAutoAcceptStatus();
    loadChampSelectConfig();
  }, []);

  const connectToLeague = async () => {
    try {
      setIsConnecting(true);
      const result = await invoke<string>('connect_to_league');
      
      setIsConnected(true);
      setConnectionStatus(result);
      addLogEntry(result, 'success');
      
      await loadAvailableChampions();
      
    } catch (error) {
      setIsConnected(false);
      setConnectionStatus('Connection failed');
      addLogEntry(`Connection failed: ${error}`, 'error');
    } finally {
      setIsConnecting(false);
    }
  };

  const loadAvailableChampions = async () => {
    try {
      const champions = await invoke<string[]>('get_all_champions');
      setAvailableChampions(champions);
    } catch (error) {
      console.error('Failed to load champions:', error);
    }
  };

  const loadChampSelectConfig = async () => {
    try {
      const savedConfig = await invoke<ChampSelectConfig>('get_champ_select_config');
      setConfig(savedConfig);
    } catch (error) {
      console.error('Failed to load config:', error);
    }
  };

  const saveChampSelectConfig = async () => {
    try {
      await invoke('update_champ_select_config', {
        autoPickEnabled: config.auto_pick_enabled,
        autoBanEnabled: config.auto_ban_enabled,
        pickPriority: config.pick_priority,
        banPriority: config.ban_priority
      });
      addLogEntry('Settings saved successfully', 'success');
      setShowSettings(false);
    } catch (error) {
      addLogEntry(`Failed to save settings: ${error}`, 'error');
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
        setMonitoringStatus('Monitoring for matches, picks & bans...');
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
        setMonitoringStatus('Monitoring for matches, picks & bans...');
      }
    } catch (error) {
      console.error('Failed to check auto-accept status:', error);
    }
  };

  const updatePickPriority = (index: number, champion: string) => {
    const newPicks = [...config.pick_priority];
    newPicks[index] = champion;
    setConfig({ ...config, pick_priority: newPicks });
  };

  const updateBanPriority = (index: number, champion: string) => {
    const newBans = [...config.ban_priority];
    newBans[index] = champion;
    setConfig({ ...config, ban_priority: newBans });
  };

  return (
    <div className="app">
      <div className="container">
        <div className="settings-icon" onClick={() => setShowSettings(true)}>
          ⚙️
        </div>
        
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

        {/* Champion Select Status */}
        <div className="status-card">
          <div className="status-indicator">
            <div className={`status-dot ${config.auto_pick_enabled ? 'monitoring' : ''}`}></div>
            <strong>Auto-Pick: {config.auto_pick_enabled ? 'Enabled' : 'Disabled'}</strong>
          </div>
          <div className="status-text">
            {config.auto_pick_enabled ? `Priority: ${config.pick_priority.join(', ')}` : 'Configure in settings'}
          </div>
        </div>

        <div className="status-card">
          <div className="status-indicator">
            <div className={`status-dot ${config.auto_ban_enabled ? 'monitoring' : ''}`}></div>
            <strong>Auto-Ban: {config.auto_ban_enabled ? 'Enabled' : 'Disabled'}</strong>
          </div>
          <div className="status-text">
            {config.auto_ban_enabled ? `Priority: ${config.ban_priority.join(', ')}` : 'Configure in settings'}
          </div>
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

        {/* Settings Modal */}
        {showSettings && (
          <div className="modal-overlay" onClick={() => setShowSettings(false)}>
            <div className="modal-content" onClick={(e) => e.stopPropagation()}>
              <div className="modal-header">
                <h2>Champion Select Settings</h2>
                <button className="modal-close" onClick={() => setShowSettings(false)}>×</button>
              </div>
              
              <div className="modal-body">
                {/* Auto-Pick Settings */}
                <div className="setting-section">
                  <div className="setting-header">
                    <label className="checkbox-container">
                      <input
                        type="checkbox"
                        checked={config.auto_pick_enabled}
                        onChange={(e) => setConfig({ ...config, auto_pick_enabled: e.target.checked })}
                      />
                      <span className="checkmark"></span>
                      Enable Auto-Pick
                    </label>
                  </div>
                  
                  <div className="priority-list">
                    <h4>Pick Priority (1st → 2nd → 3rd)</h4>
                    {config.pick_priority.map((champion, index) => (
                      <div key={index} className="priority-item">
                        <span className="priority-number">{index + 1}.</span>
                        <select
                          value={champion}
                          onChange={(e) => updatePickPriority(index, e.target.value)}
                          className="champion-select"
                        >
                          <option value="">Select Champion</option>
                          {availableChampions.map((champ) => (
                            <option key={champ} value={champ}>{champ}</option>
                          ))}
                        </select>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Auto-Ban Settings */}
                <div className="setting-section">
                  <div className="setting-header">
                    <label className="checkbox-container">
                      <input
                        type="checkbox"
                        checked={config.auto_ban_enabled}
                        onChange={(e) => setConfig({ ...config, auto_ban_enabled: e.target.checked })}
                      />
                      <span className="checkmark"></span>
                      Enable Auto-Ban
                    </label>
                  </div>
                  
                  <div className="priority-list">
                    <h4>Ban Priority (1st → 2nd → 3rd)</h4>
                    {config.ban_priority.map((champion, index) => (
                      <div key={index} className="priority-item">
                        <span className="priority-number">{index + 1}.</span>
                        <select
                          value={champion}
                          onChange={(e) => updateBanPriority(index, e.target.value)}
                          className="champion-select"
                        >
                          <option value="">Select Champion</option>
                          {availableChampions.map((champ) => (
                            <option key={champ} value={champ}>{champ}</option>
                          ))}
                        </select>
                      </div>
                    ))}
                  </div>
                </div>
              </div>

              <div className="modal-footer">
                <button className="btn btn-secondary" onClick={() => setShowSettings(false)}>
                  Cancel
                </button>
                <button className="btn btn-primary" onClick={saveChampSelectConfig}>
                  Save Settings
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default App;