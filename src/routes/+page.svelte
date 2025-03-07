<!-- Pagin principal para interacturar -->

<script lang="ts">
    //1. Recuperar keri ID si existe
    //2. Si no existe mostrar boton para evento de incepcion

    import { invoke } from "@tauri-apps/api/core";

    let name = $state("");
    let greetMsg = $state("");
    let keriMsg = $state("");

    async function loadKeri() {
        await invoke("get_keri_id");
    }

    async function greet(event: Event) {
        event.preventDefault();
        // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
        greetMsg = await invoke("greet", { name });
    }

    async function inceptionEvent(event: Event) {
        event.preventDefault();

        keriMsg = await invoke("keri_inception", {});
    }
</script>

<main class="container">
    <h1>Keri Kore App</h1>

    <form class="row" onsubmit={inceptionEvent}>
        <button type="submit">Inception Envent</button>
    </form>
    <p>{keriMsg}</p>
</main>
