<script setup lang="ts">
import {onMounted, ref} from "vue";
import {getPlaces, type PlaceWithRating} from "@/api/api";

const places = ref([] as PlaceWithRating[]);

onMounted(async () => {
  places.value = await getPlaces();
})
</script>

<template>
  <ul>
    <li v-for="place in places" :key="place.place.id">
      {{place.place.id}} - {{place.place.name}} - {{place.own_rating}} - {{place.average_rating.toFixed(1)}}
    </li>
  </ul>

</template>

<style scoped>

</style>