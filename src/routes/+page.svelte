<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  let connectionString = "udp://127.0.0.1:14551";
  let targetSystem = 1;
  let targetComponent = 0;
  let messageId = 269;
  let connectionStatus = null;

  // Store responses as an array
  let responses = [];

  // URL for the logo image
  let logoUrl = '../../avalor_logo.png';

  // Load saved connection string if available
  onMount(() => {
    const savedString = localStorage.getItem("savedConnectionString");
    if (savedString) {
      connectionString = savedString;
    }
  });

  async function checkConnection() {
    try {
      await invoke('create_connection', { connectionString });
      connectionStatus = true;
      localStorage.setItem("savedConnectionString", connectionString);
      addResponse("Connection successful");
    } catch (error) {
      connectionStatus = false;
      addResponse("Connection failed");
    }
  }

  async function sendMavlinkMessage() {
    const storedConnectionString = localStorage.getItem("savedConnectionString") || connectionString;
    const timestamp = new Date().toLocaleString();

    try {
      const message = await invoke('send_mavlink_message', {
        targetSystem,
        targetComponent,
        messageId,
        connectionString: storedConnectionString
      });
      addResponse(`Message sent -> Response: ${message}`, timestamp);
    } catch (error) {
      addResponse(`Message sent -> Error: ${error.message}`, timestamp);
    }
  }

  // Function to add a response with timestamp to the responses list
  function addResponse(text, timestamp = new Date().toLocaleString()) {
    responses = [{ text, timestamp }, ...responses];
  }
</script>

<img src={logoUrl} alt="Logo" class="logo" />

<div class="container">
  <!-- Left Column: Form -->
  <div class="form-section">
    <h1>Drone MAVLink Control</h1>

    <label>
      Connection String:
      <input type="text" bind:value={connectionString} placeholder="Enter connection string" />
      <button on:click={checkConnection}>Check Connection</button>
    </label>
    <span>
      {#if connectionStatus === true}
        <span style="color: green;">● Connected</span>
      {:else if connectionStatus === false}
        <span style="color: red;">● Not Connected</span>
      {/if}
    </span>

    <!-- Divider -->
    <hr class="divider" />

    <!-- MAVLink Command Form -->
    <form on:submit|preventDefault={sendMavlinkMessage}>
      <label>
        Target System:
        <input type="number" bind:value={targetSystem} min="1" />
      </label>
      <label>
        Target Component:
        <input type="number" bind:value={targetComponent} min="0" />
      </label>
      <label>
        Message ID:
        <input type="number" bind:value={messageId} min="1" />
      </label>
      <button type="submit">Send Message</button>
    </form>
  </div>

  <!-- Right Column: Console Output -->
  <div class="console-section">
    <p>Console Output:</p>
    <ul>
      {#each responses.reverse() as { text, timestamp }}
        <li><strong>{timestamp}:</strong> {text}</li>
      {/each}
    </ul>
  </div>
</div>

<style>
  /* Dark blue background for the whole app */
  :global(body) {
    background-color: #001f3f;
    color: white;
    font-family: Arial, sans-serif;
  }

  h1 {
    text-align: left;
    color: #ffffff;
    padding: 20px 0;
  }

  label {
    padding-top: 10px;
  }

  /* Container for two columns */
  .container {
    display: flex;
    gap: 20px;
    padding: 0 5%;
  }

  /* Left Column Styling */
  .form-section {
    width: 33%;
    display: flex;
    flex-direction: column;
    gap: 15px;
    padding: 20px;
    background-color: #003366;
    border-radius: 8px;
  }

  .form-section label {
    display: flex;
    flex-direction: column;
    font-weight: bold;
  }

  /* Logo Styling */
  .logo {
    max-width: 100%;
    height: auto;
    margin: 0 auto 15px auto;
    padding: 20px;
    display: block;
  }

  /* Divider */
  .divider {
    border: 1px solid #666;
    margin: 15px 0;
  }

  /* Right Column Styling: Console Output */
  .console-section {
    width: 66%;
    background-color: black;
    color: white;
    padding: 20px;
    border-radius: 8px;
    font-family: monospace;
  }

  .console-section p {
    margin-top: 0;
    font-weight: bold;
    color: #ffffff;
  }

  .console-section ul {
    padding-left: 15px;
    list-style-type: none;
  }

  .console-section li {
    margin-bottom: 10px;
  }

  /* Button Styling */
  button {
    margin-top: 5px;
    padding: 8px 12px;
    font-weight: bold;
    cursor: pointer;
    border: none;
    border-radius: 4px;
    background-color: #1e90ff;
    color: white;
    transition: background-color 0.3s;
  }

  button:hover {
    background-color: #3aa7ff;
  }

  /* Input Styling */
  input[type="text"], input[type="number"] {
    padding: 8px;
    border: 1px solid #ccc;
    border-radius: 4px;
    background-color: #f0f8ff;
    color: black;
    font-size: 14px;
    margin-top: 5px;
  }

  input[type="text"]:focus, input[type="number"]:focus {
    outline: none;
    border-color: #1e90ff;
  }
</style>
