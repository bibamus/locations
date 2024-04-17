<script setup lang="ts">
import {onMounted, ref} from "vue";
import {getPlaces, type PlaceWithRating} from "@/api/api";


const headers = ref([
  {title: "Name", value: "place.name"},
  {title: "Maps Link", value: "place.maps_link"},
  {
    title: "Average Rating",
    key: "average_rating",
    value: (item: PlaceWithRating) => item.average_rating.toFixed(1)
  },
  {title: "Own Rating", value: "own_rating"},
  {title: 'Actions', key: 'actions', sortable: false}
])

const placesWithRating = ref([] as PlaceWithRating[]);


onMounted(async () => {
  let result = await getPlaces();
  placesWithRating.value = result;
})
</script>

<template>
  <v-data-table
      items-per-page="-1"
      :headers="headers"
      :items="placesWithRating">
    <template v-slot:top>
      <v-toolbar flat>
        <v-toolbar-title>Places</v-toolbar-title>
        <v-dialog>
          <template v-slot:activator="{ props }">
            <v-btn class="mb-2" color="primary" dark v-bind="props">
              <v-icon icon="mdi-plus"></v-icon>
              New Place
            </v-btn>
          </template>
          <v-card>
            <v-card-title>
              <span class="text-h5">New Place</span>
            </v-card-title>
          </v-card>

        </v-dialog>
      </v-toolbar>
    </template>
    <template v-slot:[`item.actions`]="{ item }">
      <v-icon icon="mdi-pencil">
      </v-icon>
    </template>
    <template v-slot:bottom></template>
  </v-data-table>

</template>

<style scoped>

</style>