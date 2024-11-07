<script>
  // @ts-nocheck

  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  let connectionString = "udpin:172.26.96.1:14550";
  let targetSystem = 1;
  let targetComponent = 1;
  let messageId = 22;
  let connectionStatus = null;
  let responses = [];
  let debugResponses = [];
  let debugFilterText = ""; // Text to filter messages

  // MAVLink parameters
  let param1 = 0.0;
  let param2 = 0.0;
  let param3 = 0.0;
  let param4 = 0.0;
  let param5 = 0.0;
  let param6 = 0.0;
  let param7 = 0.0;

  async function backendSetup() {
    try {
      const response = await invoke("my_custom_command");
      console.log("Response from Rust:", response);
    } catch (error) {
      console.error("Failed to invoke Rust function:", error);
    }
  }

  onMount(() => {
    const savedString = localStorage.getItem("savedConnectionString");
    if (savedString) {
      connectionString = savedString;
    }

    // Listen for "log" events from the backend and add them to responses
    listen("log", (event) => {
      addResponse(event.payload, responses); // Append the payload from the log event
    });
    // Listen for "log" events from the backend and add them to responses
    listen("debug_log", (event) => {
      addResponse(event.payload, debugResponses); // Append the payload from the log event
    });

    backendSetup();
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
      addResponse("Connection successful", responses);
    } catch (error) {
      connectionStatus = false;
      addResponse("Connection failed", responses);
    }
  }

  function addResponse(
    text,
    responseArray,
    timestamp = new Date().toLocaleString()
  ) {
    responseArray.unshift({ text, timestamp });
  }

  function filteredResponses(responseArray) {
    // Filter messages based on the search bar input
    return responseArray.filter((response) =>
      response.text.toLowerCase().includes(filterText.toLowerCase())
    );
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

<div class="header">
  <img src={logoUrl} alt="Logo" class="logo" />
</div>

<div class="container">
  <!-- Left Column: Connection and Main Form -->
  <div class="form-section">
    <h1>Drone MAVLink Control</h1>
    <div class="form-group">
      <label for="con">Connection String:</label>
      <input
        id="con"
        type="text"
        bind:value={connectionString}
        placeholder="Enter connection string"
      />
    </div>
    <button class="full-width-button" on:click={checkConnection}>
      Check Connection
    </button>

    <span>
      {#if connectionStatus === true}
        <span style="color: green;">● Connected</span>
      {:else if connectionStatus === false}
        <span style="color: red;">● Not Connected</span>
      {/if}
    </span>
    <hr class="divider" />

    <form on:submit|preventDefault={sendMavlinkMessage}>
      <div class="form-group">
        <label for="a">Target System:</label>
        <input id="a" type="number" bind:value={targetSystem} min="0" />
      </div>
      <div class="form-group">
        <label for="b">Target Component:</label>
        <input id="b" type="number" bind:value={targetComponent} min="0" />
      </div>
      <div class="form-group">
        <label for="c">Message ID:</label>
        <input id="c" type="number" bind:value={messageId} min="1" />
      </div>
      <button class="full-width-button" type="submit">Send Message</button>
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

  <!-- Right Columns: Console Output and Debug Output -->
  <div class="output-container">
    <div class="console-section">
      <p>Console Output:</p>
      <ul>
        {#each responses.reverse() as { text, timestamp }}
          <li><strong>{timestamp}:</strong> {text}</li>
        {/each}
      </ul>
    </div>

    <div class="debug-section">
      <p>Debug Output:</p>
      <ul>
        {#each filteredResponses(debugResponses, debugFilterText) as { text, timestamp }}
          <li><strong>{timestamp}:</strong> {text}</li>
        {/each}
      </ul>
      <input
        type="text"
        bind:value={debugFilterText}
        placeholder="Filter debug messages"
      />
    </div>
  </div>
</div>

<style>
  .form-section {
    background-color: #003366;
    padding: 20px;
    border-radius: 8px;
    color: white;
    display: flex;
    flex-direction: column;
    gap: 15px;
    text-align: left;
  }

  /* Styling for each label-input group */
  .form-group {
    display: grid;
    grid-template-columns: 150px 1fr; /* Set column widths */
    align-items: center;
    gap: 10px;
    margin-bottom: 10px;
  }
  .header {
    display: flex;
    justify-content: center;
    padding: 20px 0;
    background-color: #001f3f;
  }
  /* Main container to hold all columns */
  .container {
    display: flex;
    gap: 20px;
    padding: 0 5%;
    height: 90vh; /* Full viewport height */
    background-color: #001f3f;
  }

  .full-width-button {
    width: 100%; /* Full width */
    margin-top: 10px;
    font-weight: bold;
    cursor: pointer;
    border: none;
    border-radius: 4px;
    background-color: #1e90ff;
    color: white;
    transition: background-color 0.3s;
  }

  /* Form and parameter sections */
  .form-section,
  .param-section {
    flex: 1;
    background-color: #003366;
    padding: 20px;
    border-radius: 8px;
    color: white;
    display: flex;
    flex-direction: column;
    gap: 15px;
    align-items: left;
  }

  .form-section {
    text-align: left; /* Left-align text in the form section */
  }

  .param-section {
    background-color: #004080;
  }

  /* Output container to hold console and debug sections */
  .output-container {
    display: flex;
    flex-direction: column;
    flex: 2; /* Wider than the form and parameter sections */
    gap: 20px;
  }

  .console-section,
  .debug-section {
    background-color: black;
    padding: 20px;
    border-radius: 8px;
    color: white;
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%; /* Make each section occupy full height */
  }

  .console-section p,
  .debug-section p {
    margin-top: 0;
    font-weight: bold;
    color: #ffffff;
  }

  .console-section ul,
  .debug-section ul {
    flex-grow: 1;
    overflow-y: auto;
    list-style-type: none;
    padding-left: 15px;
  }

  .console-section li,
  .debug-section li {
    margin-bottom: 10px;
  }

  .debug-section input {
    margin-top: auto; /* Keep input at the bottom */
  }

  /* General input styling */
  input[type="text"],
  input[type="number"] {
    padding: 8px;
    border: 1px solid #ccc;
    border-radius: 4px;
    background-color: #f0f8ff;
    color: black;
    font-size: 14px;
  }

  input[type="text"]:focus,
  input[type="number"]:focus {
    outline: none;
    border-color: #1e90ff;
  }
</style>
