<script setup lang="ts">
import {onMounted, ref, watch} from "vue";
import {createPlace, getPlaces, type Place, type PlaceWithRating, ratePlace, updatePlace} from "@/api/api";


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
const currentEditItem = ref({} as Place);

const dialog = ref(false)


onMounted(async () => {
  placesWithRating.value = await getPlaces();
})

watch(dialog, (newVal) => {
  if (!newVal) {
    currentEditItem.value = {} as Place;
  }
});

function editItem(item: Place) {
  currentEditItem.value = Object.assign({}, item);
  dialog.value = true;
}

async function saveItem() {
  let data = currentEditItem.value;
  if (data.id == null) {
    await createPlace(data);
  } else {
    await updatePlace(data);
  }
  dialog.value = false;
  placesWithRating.value = await getPlaces();
}

async function rate(placeId: string, value: number) {
  await ratePlace(placeId, value);
  placesWithRating.value = await getPlaces();
}

</script>

<template>
  <v-data-table
      items-per-page="-1"
      :headers="headers"
      :items="placesWithRating">
    <template v-slot:top>
      <v-toolbar flat>
        <v-toolbar-title>Places</v-toolbar-title>
        <v-dialog v-model="dialog">
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
            <v-container>
              <v-row>
                <v-col>
                  <v-text-field
                      v-model="currentEditItem.name"
                      label="Name"
                  ></v-text-field>
                </v-col>
                <v-col>
                  <v-text-field
                      v-model="currentEditItem.maps_link"
                      label="Maps Link"
                  ></v-text-field>
                </v-col>
              </v-row>
              <v-row>
                <v-col>
                  <v-btn @click="dialog = false">Cancel</v-btn>
                </v-col>
                <v-col>
                  <v-btn @click="saveItem()" color="primary">Save</v-btn>
                </v-col>
              </v-row>
            </v-container>
          </v-card>

        </v-dialog>
      </v-toolbar>
    </template>
    <template v-slot:[`item.own_rating`]="{ item }">
      <v-rating v-model="item.own_rating" active-color="#FFEB3B" length="5" hover @update:model-value="(value: number|string) => rate(item.place.id, value as number)">
      </v-rating>
    </template>
    <template v-slot:[`item.actions`]="{ item }">
      <v-icon icon="mdi-pencil" @click="editItem(item.place)">
      </v-icon>
    </template>
    <template v-slot:bottom></template>
  </v-data-table>

</template>

<style scoped>

</style>