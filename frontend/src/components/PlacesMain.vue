<script setup lang="ts">
import {onMounted, ref} from "vue";
import {getPlaces, type Place} from "@/api/api";

const places = ref([] as Place[]);

onMounted(async () => {
  places.value = await getPlaces();
})
</script>

<template>
  <ul>
    <li v-for="place in places" :key="place.id">
      {{place.id}} - {{place.name}}
    </li>
  </ul>

</template>

<style scoped>

</style>