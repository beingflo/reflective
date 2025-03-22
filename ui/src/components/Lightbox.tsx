import { useNavigate } from '@solidjs/router';
import { createEffect, type Component } from 'solid-js';
import { useStore } from '../store';

export type LightboxProps = {
  imageId: string;
};

const Lightbox: Component<LightboxProps> = (props: LightboxProps) => {
  const [state, { setImages }] = useStore();

  // Fetch images in case of reload
  createEffect(async () => {
    if (state.images.length === 0) {
      const response = await fetch('/api/images', {
        headers: {
          'content-type': 'application/json',
        },
      }).then((response) => response.json());

      setImages(response);
    }
  });

  createEffect(() => {
    const currentIndex = state.images.indexOf(props.imageId);
    const nextImage = state.images[currentIndex + 1];
    const lastImage = state.images[currentIndex - 1];

    if (nextImage) {
      new Image().src = `/api/images/${nextImage?.id}?quality=medium`;
    }
    if (lastImage) {
      new Image().src = `/api/images/${lastImage?.id}?quality=medium`;
    }
  });

  return (
    <div class="fixed bg-white flex w-full h-screen p-2 md:p-8 justify-center">
      <img
        class="h-full w-full object-contain"
        src={`/api/images/${props.imageId}?quality=medium`}
      />
    </div>
  );
};

export default Lightbox;
