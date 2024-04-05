<script setup lang="ts">
import Places from "@/components/Places.vue";
import {onMounted, ref} from "vue";
import {login, getToken} from "./auth/auth"

const authenticated = ref(false)

onMounted(async () => {
  console.log('App.vue')
  await login();
  authenticated.value = true
  const token = await getToken();
  // console.log(token)
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