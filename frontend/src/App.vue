<script setup lang="ts">
import Places from "@/components/PlacesMain.vue";
import {onMounted, ref} from "vue";
import {login} from "./auth/auth"

const authenticated = ref(false)

onMounted(async () => {
  await login();
  authenticated.value = true
})

</script>


<template>
  <template v-if="authenticated">
    <header>
      <h1>Places</h1>
    </header>
    <main>
      <Places/>
    </main>
  </template>
  <template v-else>
    <main>
      <p>Authenticating...</p>
    </main>
  </template>
</template>