import type { Component } from 'solid-js';

const NotFound: Component = () => {
  return (
    <div class="mx-auto flex flex-col w-1/2 mt-24">
      <p class="text-4xl mx-auto md:text-6xl mb-4 text-black dark:text-white font-extrabold">
        404 Not Found
      </p>
    </div>
  );
};

export default NotFound;
