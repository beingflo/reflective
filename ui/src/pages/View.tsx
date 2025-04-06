import { useNavigate } from '@solidjs/router';
import {
  type Component,
  createEffect,
  createMemo,
  createSignal,
  For,
  onCleanup,
  Show,
} from 'solid-js';
import { useStore } from '../store';
import Lightbox from '../components/Lightbox';
import { tinykeys } from 'tinykeys';
import { validateEvent } from '../utils';
import { createVisibilityObserver } from '@solid-primitives/intersection-observer';

const View: Component = () => {
  const [state, { setImages }] = useStore();
  const [openImage, setOpenImage] = createSignal('');
  const [tagMode, setTagMode] = createSignal(false);
  const [searchMode, setSearchMode] = createSignal(false);
  const [searchTerm, setSearchTerm] = createSignal('');
  const [newTagValue, setNewTagValue] = createSignal('');
  const [lastSelectedImage, setLastSelectedImage] = createSignal();
  const [selectedImages, setSelectedImages] = createSignal([]);
  const navigate = useNavigate();
  let searchInputRef;

  const goToNextImage = () => {
    if (!openImage()) return;
    const currentIndex = state.images.findIndex(
      (image) => image.id === openImage(),
    );
    const nextImage = state.images[currentIndex + 1];

    if (nextImage) {
      setOpenImage(nextImage?.id);
    }
  };

  const goToLastImage = () => {
    if (!openImage()) return;
    const currentIndex = state.images.findIndex(
      (image) => image.id === openImage(),
    );
    const lastImage = state.images[currentIndex - 1];

    if (lastImage) {
      setOpenImage(lastImage?.id);
    }
  };

  const onClickImage = (imageId: string, e: MouseEvent) => {
    if (tagMode()) {
      if (!e.shiftKey) {
        setSelectedImages((prev) => {
          if (prev.includes(imageId)) {
            return prev.filter((id) => id !== imageId);
          } else {
            return [...prev, imageId];
          }
        });
      } else {
        let lastImageIdx = state.images.findIndex(
          (image) => image.id === lastSelectedImage(),
        );
        let currentImageIdx = state.images.findIndex(
          (image) => image.id === imageId,
        );

        if (lastImageIdx === undefined || currentImageIdx === undefined) {
          return;
        }

        let startIdx = Math.min(lastImageIdx, currentImageIdx);
        let endIdx = Math.max(lastImageIdx, currentImageIdx);

        let range = state.images
          .slice(startIdx, endIdx + 1)
          .map((image) => image.id);

        setSelectedImages((prev) => {
          let newSelected = [...new Set([...prev, ...range])];
          return newSelected;
        });
      }
      setLastSelectedImage(imageId);
    } else {
      setOpenImage(imageId);
    }
  };

  const loadImages = async () => {
    const response = await fetch('/api/images', {
      headers: {
        'content-type': 'application/json',
      },
    }).catch((error) => {
      console.error('Failed to fetch images:', error);
      throw error;
    });

    if (response.status === 401) {
      navigate('/login');
      return;
    }

    const data = await response.json();
    setImages(data);
  };

  const searchImages = async (query: string) => {
    setSearchTerm(query);
    const response = await fetch('/api/images/search', {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
      },
      body: JSON.stringify({
        query,
      }),
    }).catch((error) => {
      console.error('Failed to search images:', error);
      throw error;
    });

    if (response.status === 401) {
      navigate('/login');
      return;
    }

    const data = await response.json();
    setImages(data);
  };

  const onRemoveTag = async (tag: string) => {
    const response = await fetch('/api/tags', {
      method: 'DELETE',
      headers: {
        'content-type': 'application/json',
      },
      body: JSON.stringify({
        image_ids: selectedImages(),
        tags: [tag],
      }),
    }).catch((error) => {
      console.error('Failed to delete tag:', error);
      throw error;
    });

    if (response.status === 401) {
      navigate('/login');
      return;
    }

    if (response.status === 200) {
      loadImages();
    }
  };

  const onNewTag = async (tag: string) => {
    const response = await fetch('/api/tags', {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
      },
      body: JSON.stringify({
        image_ids: selectedImages(),
        tags: [tag],
      }),
    }).catch((error) => {
      console.error('Failed to add tag:', error);
      throw error;
    });

    if (response.status === 401) {
      navigate('/login');
      return;
    }

    if (response.status === 200) {
      setNewTagValue('');
      loadImages();
    }
  };

  const closeLightbox = () => {
    document
      .getElementById(openImage())
      ?.scrollIntoView({ behavior: 'instant', block: 'center' });
    setOpenImage('');
  };

  const cleanup = tinykeys(window, {
    ArrowRight: validateEvent(goToNextImage),
    ArrowLeft: validateEvent(goToLastImage),
    Escape: () => {
      if (tagMode()) {
        setTagMode(false);
      } else if (searchMode()) {
        setSearchMode(false);
        searchInputRef.blur();
      } else {
        closeLightbox();
      }
    },
    '$mod+e': () => {
      if (!openImage()) {
        setTagMode((prev) => !prev);
      }
    },
    '$mod+c': () => {
      setSelectedImages([]);
    },
    '$mod+k': (event) => {
      if (!openImage()) {
        event.preventDefault();
        setSearchMode((prev) => !prev);
        searchInputRef.focus();
      }
    },
    '$mod+u': validateEvent(() => navigate('/upload')),
  });

  onCleanup(cleanup);

  // Scroll to top of page on refresh
  window.onbeforeunload = function () {
    window.scrollTo(0, 0);
  };

  createEffect(async () => {
    loadImages();
  });

  const selectedImagesTags = (): Array<string> => {
    let tags = new Set();
    selectedImages().forEach((imageId) => {
      const image = state?.images?.find((img) => img?.id === imageId);
      image?.tags?.forEach((tag) => tags.add(tag));
    });

    return [...tags] as Array<string>;
  };

  let allImageTags = () => {
    let allTags = selectedImagesTags();
    selectedImages().forEach((imageId) => {
      const image = state?.images?.find((img) => img?.id === imageId);
      allTags = allTags.filter((tag) => image?.tags?.includes(tag));
    });

    return allTags;
  };

  let [numImages, setNumImages] = createSignal(10);

  let [leftImages, setLeftImages] = createSignal([]);
  let [middleImages, setMiddleImages] = createSignal([]);
  let [rightImages, setRightImages] = createSignal([]);

  createEffect(() => {
    let heightLeft = 0;
    let heightMiddle = 0;
    let heightRight = 0;

    let imagesLeft = [];
    let imagesMiddle = [];
    let imagesRight = [];

    const images = state.images.slice(0, numImages());

    images.forEach((image) => {
      const imageHeight = 1 / image.aspect_ratio;
      if (heightLeft <= heightMiddle && heightLeft <= heightRight) {
        heightLeft += imageHeight;
        imagesLeft.push(image);
      } else if (heightMiddle <= heightLeft && heightMiddle <= heightRight) {
        heightMiddle += imageHeight;
        imagesMiddle.push(image);
      } else if (heightRight <= heightLeft && heightRight <= heightMiddle) {
        heightRight += imageHeight;
        imagesRight.push(image);
      }
    });
    setLeftImages(imagesLeft);
    setMiddleImages(imagesMiddle);
    setRightImages(imagesRight);
  });

  let el: HTMLDivElement | undefined;

  const visible = createVisibilityObserver({
    threshold: 1.0,
    rootMargin: '800px',
  })(() => el);

  createEffect(() => {
    if (visible()) {
      setNumImages((prev) => prev + 20);
    }
    console.log(allImageTags());
  });

  return (
    <div>
      <Show when={state.images.length === 0}>
        <div class="flex w-full h-96">
          <div class="m-auto flex flex-col gap-4">
            <h1 class="text-4xl text-center ">No images found</h1>
            <p class="text-center">
              Press <span class="font-bold">U</span> to upload
            </p>
          </div>
        </div>
      </Show>
      <Show when={openImage()}>
        <Lightbox imageId={openImage()} />
      </Show>
      <Show when={searchMode()}>
        <div class="fixed top-0 w-full">
          <div class="flex flex-row bg-white border-b border-black rounded-sm w-full h-12">
            <div class="pr-2 border-r border-black w-60 p-2 pt-3">
              <p class="text-sm text-gray-700">
                matched images: {state.images.length}
              </p>
            </div>
            <div class="p-2 flex w-full flex-row items-start">
              <input
                ref={searchInputRef}
                value={searchTerm()}
                class="p-1.5 w-full mx-1 outline-none text-xs"
                placeholder="search"
                autofocus
                onInput={(e) => searchImages(e.currentTarget.value)}
              />
            </div>
          </div>
        </div>
      </Show>
      <Show when={tagMode()}>
        <div class="fixed bottom-0 w-full">
          <div class="flex flex-row bg-white border-t border-black rounded-sm w-full h-12">
            <div class="pr-2 border-r border-black w-60 p-2 pt-3">
              <p class="text-sm text-gray-700">
                selected images: {selectedImages().length}
              </p>
            </div>
            <div class="p-2 flex flex-row items-start overflow-x-scroll">
              <For each={selectedImagesTags()}>
                {(tag) => (
                  <div
                    class={`flex flex-row gap-2 items-center text-black rounded-md p-1 px-2 mx-1 ${
                      allImageTags().includes(tag)
                        ? 'bg-slate-300'
                        : 'bg-slate-100'
                    }`}
                  >
                    <p class="text-sm">{tag}</p>
                    <p
                      class="text-xs cursor-pointer"
                      onClick={(e) => onRemoveTag(tag)}
                    >
                      ✕
                    </p>
                  </div>
                )}
              </For>
              <form
                onSubmit={(e: SubmitEvent) => {
                  e.preventDefault();
                  onNewTag(newTagValue());
                }}
              >
                <input
                  class="p-1.5 mx-1 outline-none text-xs"
                  placeholder="new tag"
                  value={newTagValue()}
                  onInput={(e) => setNewTagValue(e.currentTarget.value)}
                />
              </form>
            </div>
          </div>
        </div>
      </Show>
      <div class="flex flex-col w-full">
        <div class="flex flex-row gap-4 p-8 pt-16 max-w-screen-2xl mx-auto">
          <div class="flex flex-col gap-4 w-1/3">
            <For each={leftImages()}>
              {(image) => (
                <div
                  class={`w-full aspect-[${image.aspect_ratio}] h-auto ${
                    selectedImages().includes(image?.id) && tagMode()
                      ? 'outline outline-3 outline-offset-2 outline-blue-600'
                      : ''
                  }`}
                >
                  <img
                    class="object-fill w-full"
                    id={image?.id}
                    onClick={(e) => onClickImage(image?.id, e)}
                    src={`/api/images/${image?.id}?quality=small`}
                  />
                </div>
              )}
            </For>
          </div>
          <div class="flex flex-col gap-4 w-1/3">
            <For each={middleImages()}>
              {(image) => (
                <div
                  class={`w-full aspect-[${image.aspect_ratio}] h-auto ${
                    selectedImages().includes(image?.id) && tagMode()
                      ? 'outline outline-3 outline-offset-2 outline-blue-600'
                      : ''
                  }`}
                >
                  <img
                    class={`object-fill w-full aspect-[${image.aspect_ratio}]`}
                    id={image?.id}
                    onClick={(e) => onClickImage(image?.id, e)}
                    src={`/api/images/${image?.id}?quality=small`}
                  />
                </div>
              )}
            </For>
          </div>
          <div class="flex flex-col gap-4 w-1/3">
            <For each={rightImages()}>
              {(image) => (
                <div
                  class={`w-full aspect-[${image.aspect_ratio}] h-auto ${
                    selectedImages().includes(image?.id) && tagMode()
                      ? 'outline outline-3 outline-offset-2 outline-blue-600'
                      : ''
                  }`}
                >
                  <img
                    class={`object-fill w-full aspect-[${image.aspect_ratio}]`}
                    id={image?.id}
                    onClick={(e) => onClickImage(image?.id, e)}
                    src={`/api/images/${image?.id}?quality=small`}
                  />
                </div>
              )}
            </For>
          </div>
        </div>
        <div ref={el} class="h-1" />
      </div>
    </div>
  );
};

export default View;
