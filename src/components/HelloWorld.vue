<template>
  <div class="card">
    <Carousel :value="products" :numVisible="3" :numScroll="3" :responsiveOptions="responsiveOptions" circular>
      <template #item="slotProps">
        <div class="border-1 surface-border border-round m-2  p-3">
          <div class="mb-3">
            <div class="relative mx-auto">
              <img loading="lazy" decoding="async" :src="slotProps.data.url" class="border-round" width="150" height="100"/>
              <Tag :value="slotProps.data.saved ? 'Сохранен' : 'Не сохранен'" :severity="getSeverity(slotProps.data.saved)" class="absolute" style="left:5px; top: 5px"/>
            </div>
          </div>
          <div class="mb-3 font-medium">{{ slotProps.data.name }}</div>
          <div class="flex justify-content-between align-items-center">
            <span>
                            <Button icon="pi pi-heart" severity="secondary" outlined />
                            <Button icon="pi pi-shopping-cart" class="ml-2"/>
                        </span>
          </div>
        </div>
      </template>
    </Carousel>
  </div>
  <span>
                            <Button @click="selectFolder()" icon="pi pi-heart" severity="secondary" outlined />
                            <Button icon="pi pi-shopping-cart" class="ml-2"/>
                        </span>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import {IPicturesResponse, ProductService} from '@/service/ProductService';
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/api/dialog';
import { appDataDir } from '@tauri-apps/api/path';
import Carousel from "primevue/carousel";
import Button from "primevue/button";
import Tag from "primevue/tag";


const selectFolder = async () => {
  const selected = await open({
    multiple: false,
    directory: true,
    defaultPath: await appDataDir(),
  });
  console.log(selected)
}

onMounted(() => {
  //sendOutput()
  const prodService = new ProductService()
  prodService.getImages().then((data) => { products.value = data
    console.log(products.value)
  });
})
/*
const output = ref("");
const outputs = ref<{timestamp: number, message: string}[]>([]);
const inputs = ref<{timestamp: number, message: string}[]>([]);

function sendOutput() {
  console.log("js: js2rs: " + output.value)
  outputs.value.push({ timestamp: Date.now(), message: output.value })
  invoke('js2rs', { message: output.value })
}

await listen('rs2js', (event) => {
  console.log("js: rs2js: " + event)
  let input: string = event.payload as string
  inputs.value.push({ timestamp: Date.now(), message: input })
})*/

const products = ref<IPicturesResponse[]>();
const responsiveOptions = ref([
  {
    breakpoint: '1400px',
    numVisible: 2,
    numScroll: 1
  },
  {
    breakpoint: '1199px',
    numVisible: 3,
    numScroll: 1
  },
  {
    breakpoint: '767px',
    numVisible: 2,
    numScroll: 1
  },
  {
    breakpoint: '575px',
    numVisible: 1,
    numScroll: 1
  }
]);

const getSeverity = (status: IPicturesResponse['saved']) => {
  switch (status) {
    case true:
      return 'success';

    case false:
      return 'danger';

    default:
      return null;
  }
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
.card {
  background: var(--surface-card);
  padding: 2rem;
  border-radius: 10px;
  margin-bottom: 1rem;
}
</style>
