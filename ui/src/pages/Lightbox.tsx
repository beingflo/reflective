import { useParams } from '@solidjs/router';
import { type Component } from 'solid-js';

const Lightbox: Component = () => {
  const { id } = useParams();

  return (
    <div class="flex w-full h-screen p-2 md:p-8 justify-center">
      <img
        class="h-full w-full object-contain"
        src={`/api/images/${id}?quality=original`}
      />
    </div>
  );
};

export default Lightbox;
