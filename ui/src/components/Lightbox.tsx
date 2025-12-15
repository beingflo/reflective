import { createEffect, Show, type Component } from 'solid-js';
import { type Image } from '../store';

export type LightboxProps = {
  imageId: string;
  images: Array<Image>;
  close: () => void;
  showMetadata: boolean;
  metadata: Object;
  originalQuality: boolean;
};

const Lightbox: Component<LightboxProps> = (props: LightboxProps) => {
  createEffect(() => {
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

  const imageUrl = () =>
    `/api/images/${props.imageId}?quality=${
      props.originalQuality ? 'original' : 'medium'
    }`;

  return (
    <div class="fixed bg-stone-100 flex w-full h-screen p-2 md:p-8 justify-center">
      <button
        onClick={props.close}
        class="md:hidden absolute top-2 right-4 p-4"
      >
        x
      </button>
      <div>
        <Show when={props.showMetadata}>
          {props.metadata?.['Make']?.replaceAll('"', '')}
          {props.metadata?.['Model']?.replaceAll('"', '')}
        </Show>
        <img class="h-full w-full object-contain" src={imageUrl()} />
      </div>
    </div>
  );
};

export default Lightbox;
