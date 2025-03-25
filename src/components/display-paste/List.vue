<script setup> 
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const paste = ref("");

const getPaste = async () => {
  paste.value = await invoke("get_clipboard");
  console.log(paste);
};

</script>

<template>
  <main class="container">
    <h1>Welcome to Tauri + Vue</h1>
    <button @click="getPaste">Get Paste</button>
    <p>{{ paste }}</p>
  </main>
</template>
