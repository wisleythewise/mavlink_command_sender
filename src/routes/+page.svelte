<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  let connectionString = "udpin:172.26.96.1:14550";
  let targetSystem = 1;
  let targetComponent = 1;
  let messageId = 22;
  let connectionStatus = null;
  let responses = [];

  // MAVLink parameters
  let param1 = 0.0;
  let param2 = 0.0;
  let param3 = 0.0;
  let param4 = 0.0;
  let param5 = 0.0;
  let param6 = 0.0;
  let param7 = 0.0;

  onMount(() => {
    const savedString = localStorage.getItem("savedConnectionString");
    if (savedString) {
      connectionString = savedString;
    }

    // Listen for "log" events from the backend and add them to responses
    listen("log", (event) => {
      addResponse(event.payload); // Append the payload from the log event
    });
  });

  function format_message(
    targetSystem,
    targetComponent,
    messageId,
    param1,
    param2,
    param3,
    param4,
    param5,
    param6,
    param7
  ) {
    return `Sent Message:
  - Target System: ${targetSystem}
  - Target Component: ${targetComponent}
  - Message ID: ${messageId}
  - Params: [${param1}, ${param2}, ${param3}, ${param4}, ${param5}, ${param6}, ${param7}]
  `;
  }

  async function checkConnection() {
    try {
      await invoke("create_connection", { connectionString });
      connectionStatus = true;
      localStorage.setItem("savedConnectionString", connectionString);
      addResponse("Connection successful");
    } catch (error) {
      connectionStatus = false;
      addResponse("Connection failed");
    }
  }

  function addResponse(text, timestamp = new Date().toLocaleString()) {
    responses = [{ text, timestamp }, ...responses];
  }

  async function sendMavlinkMessage() {
    const storedConnectionString =
      localStorage.getItem("savedConnectionString") || connectionString;
    const timestamp = new Date().toLocaleString();

    try {
      const message = await invoke("send_mavlink_message", {
        targetSystem,
        targetComponent,
        commandId: messageId,
        param1,
        param2,
        param3,
        param4,
        param5,
        param6,
        param7,
        connectionString: storedConnectionString,
      });

      // Format the sent message details and add it to responses
      const formattedMessage = format_message(
        targetSystem,
        targetComponent,
        messageId,
        param1,
        param2,
        param3,
        param4,
        param5,
        param6,
        param7
      );
      addResponse(`${formattedMessage}\nResponse: ${message}`, timestamp);
    } catch (error) {
      const formattedMessage = format_message(
        targetSystem,
        targetComponent,
        messageId,
        param1,
        param2,
        param3,
        param4,
        param5,
        param6,
        param7
      );
      addResponse(`${formattedMessage}\n  Error: ${error}`, timestamp);
    }
  }

  let logoUrl = "../../avalor_logo.png";
</script>

<img src={logoUrl} alt="Logo" class="logo" />

<div class="container">
  <!-- Left Column: Connection and Main Form -->
  <div class="form-section">
    <h1>Drone MAVLink Control</h1>
    <label>
      Connection String:
      <input
        type="text"
        bind:value={connectionString}
        placeholder="Enter connection string"
      />
      <button on:click={checkConnection}>Check Connection</button>
    </label>
    <span>
      {#if connectionStatus === true}
        <span style="color: green;">● Connected</span>
      {:else if connectionStatus === false}
        <span style="color: red;">● Not Connected</span>
      {/if}
    </span>
    <hr class="divider" />
    <form on:submit|preventDefault={sendMavlinkMessage}>
      <label
        >Target System: <input
          type="number"
          bind:value={targetSystem}
          min="0"
        /></label
      >
      <label
        >Target Component: <input
          type="number"
          bind:value={targetComponent}
          min="0"
        /></label
      >
      <label
        >Message ID: <input
          type="number"
          bind:value={messageId}
          min="1"
        /></label
      >
      <button type="submit">Send Message</button>
    </form>
  </div>

  <!-- Middle Column: MAVLink Parameters -->
  <div class="param-section">
    <h2>Command Parameters</h2>
    <input type="number" bind:value={param1} placeholder="Param 1" />
    <input type="number" bind:value={param2} placeholder="Param 2" />
    <input type="number" bind:value={param3} placeholder="Param 3" />
    <input type="number" bind:value={param4} placeholder="Param 4" />
    <input type="number" bind:value={param5} placeholder="Param 5" />
    <input type="number" bind:value={param6} placeholder="Param 6" />
    <input type="number" bind:value={param7} placeholder="Param 7" />
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
  /* Styling remains the same as before */
  .container {
    display: flex;
    gap: 20px;
    padding: 0 5%;
  }
  .form-section,
  .console-section {
    flex: 1;
    background-color: #003366;
    padding: 20px;
    border-radius: 8px;
    color: white;
  }
  .console-section {
    background-color: black;
  }

  /* Styling for the parameter section */
  .param-section {
    flex: 1;
    background-color: #004080;
    padding: 20px;
    padding-right: 50px;
    border-radius: 8px;
    color: white;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  /* Style each parameter input */
  .param-section input[type="number"] {
    padding: 8px;
    border: 1px solid #ccc;
    border-radius: 4px;
    background-color: #f0f8ff;
    color: black;
    font-size: 14px;
    width: 100%;
  }

  /* Placeholder text color for better visibility */
  .param-section input::placeholder {
    color: #666;
  }

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
  input[type="text"],
  input[type="number"] {
    padding: 8px;
    border: 1px solid #ccc;
    border-radius: 4px;
    background-color: #f0f8ff;
    color: black;
    font-size: 14px;
    margin-top: 5px;
  }

  input[type="text"]:focus,
  input[type="number"]:focus {
    outline: none;
    border-color: #1e90ff;
  }
</style>
