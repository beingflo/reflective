import { createSignal, type Component, onMount } from 'solid-js';

const Upload: Component = () => {
  const [content, setContent] = createSignal('');
  let ref: HTMLInputElement;

  onMount(() => {
    ref.addEventListener('change', () => {
      const reader = new FileReader();
      reader.onload = (evt) => {
        console.log(evt.target.result);
      };
      const numFiles = ref.files.length;
      [...Array(numFiles).keys()].forEach((i: number) => {
        reader.readAsText(ref.files.item(i));
      });
    });
  });

  return (
    <div class="mx-auto flex flex-col w-1/2 min-w-96 pt-12">
      <div class="flex flex-row gap-4 items-baseline">
        <p class="text-4xl md:text-6xl mb-4 text-black dark:text-white font-extrabold">
          Upload
        </p>
      </div>
      <label
        class="mt-8 text-center px-4 rounded-sm bg-white border border-black py-2 uppercase text-black hover:bg-gray-100 hover:shadow-none focus:outline-none hover:cursor-pointer"
        for="file"
      >
        Select files
      </label>
      <input ref={ref} type="file" id="file" class="hidden" multiple />
    </div>
  );
};

export default Upload;
