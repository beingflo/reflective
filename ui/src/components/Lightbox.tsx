import { createEffect, type Component } from 'solid-js';
import { useStore, type Image } from '../store';

export type LightboxProps = {
  imageId: string;
  images: Array<Image>;
  close: () => void;
  originalQuality: boolean;
};

const Lightbox: Component<LightboxProps> = (props: LightboxProps) => {
  const [, { setImages }] = useStore();

  // Fetch images in case of reload
  createEffect(async () => {
    if (props.images.length === 0) {
      const response = await fetch('/api/images', {
        headers: {
          'content-type': 'application/json',
        },
      }).then((response) => response.json());

      setImages(response);
    }
  });

  createEffect(() => {
    // todo, shouldn't be state.images but images from list above
    const currentIndex = props.images.findIndex(
      (image) => image.id === props.imageId,
    );
    const nextImage = props.images[currentIndex + 1];
    const lastImage = props.images[currentIndex - 1];

    if (nextImage) {
      new Image().src = `/api/images/${nextImage?.id}?quality=${
        props.originalQuality ? 'original' : 'medium'
      }`;
    }
    if (lastImage) {
      new Image().src = `/api/images/${lastImage?.id}?quality=${
        props.originalQuality ? 'original' : 'medium'
      }`;
    }
  });

  return (
    <div class="fixed bg-stone-100 flex w-full h-screen p-2 md:p-8 justify-center">
      <button
        onClick={props.close}
        class="md:hidden absolute top-2 right-4 p-4"
      >
        x
      </button>
      <img
        class="h-full w-full object-contain"
        src={`/api/images/${props.imageId}?quality=${
          props.originalQuality ? 'original' : 'medium'
        }`}
      />
    </div>
  );
};

export default Lightbox;
