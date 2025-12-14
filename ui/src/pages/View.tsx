import { useNavigate } from '@solidjs/router';
import {
  type Component,
  createEffect,
  createResource,
  createSignal,
  For,
  onCleanup,
  Show,
} from 'solid-js';
import { useStore } from '../store';
import Lightbox from '../components/Lightbox';
import { tinykeys } from 'tinykeys';
import { validateEvent } from '../utils';
import { debounce } from '@solid-primitives/scheduled';
import { createMediaQuery } from '@solid-primitives/media';

const View: Component = () => {
  const [state, { setImages, appendImages }] = useStore();
  const [openImage, setOpenImage] = createSignal('');
  const [tagMode, setTagMode] = createSignal(false);
  const [searchMode, setSearchMode] = createSignal(false);
  const [emptyTagMode, setEmptyTagMode] = createSignal(false);
  const [searchTerm, setSearchTerm] = createSignal('');
  const [newTagValue, setNewTagValue] = createSignal('');
  const [lastSelectedImage, setLastSelectedImage] = createSignal();
  const [selectedImages, setSelectedImages] = createSignal([]);
  const [originalQuality, setOriginalQuality] = createSignal(false);
  const [page, setPage] = createSignal(1);
  const navigate = useNavigate();

  const isMobile = createMediaQuery('(max-width: 767px)');

  const searchDebounced = debounce((term: string) => {
    setImages([]);
    setPage(1);
    setSearchTerm(term);
  }, 250);

  const fetchImages = async ({
    query,
    page,
  }: {
    query: String;
    page: number;
  }) => {
    const response = await fetch('/api/images/search', {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
      },
      body: JSON.stringify({
        query: query,
        page: page,
        limit: 40,
      }),
    }).catch((error) => {
      console.error('Failed to search images:', error);
      throw error;
    });

    if (response.status === 401) {
      navigate('/login/');
      return;
    }

    return response.json();
  };

  const [data] = createResource(
    () => ({ query: searchTerm(), page: page() }),
    fetchImages,
  );

  createEffect(() => {
    if (!data.loading) {
      appendImages(data().images);
    }
  });

  const [qualityHint, setQualityHint] = createSignal(false);

  const showQualityHint = () => {
    setQualityHint(true);
    setTimeout(() => setQualityHint(false), 500);
  };

  let imagesObserver: IntersectionObserver;
  let loadingObserver: IntersectionObserver;

  let searchInputRef;

  const images = () => {
    if (emptyTagMode()) {
      return state.images.filter((image) => image.tags.length === 0);
    }

    return state.images;
  };

  const goToNextImage = () => {
    if (!openImage()) return;

    const currentIndex = images().findIndex(
      (image) => image.id === openImage(),
    );

    const nextImage = images()[currentIndex + 1];

    if (nextImage) {
      setOpenImage(nextImage?.id);
    }

    if (currentIndex >= images().length - 10) {
      if (!data.loading && data().has_more) {
        setPage((page) => page + 1);
      }
    }
  };

  const goToLastImage = () => {
    if (!openImage()) return;
    const currentIndex = images().findIndex(
      (image) => image.id === openImage(),
    );
    const lastImage = images()[currentIndex - 1];

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
        let lastImageIdx = images().findIndex(
          (image) => image.id === lastSelectedImage(),
        );
        let currentImageIdx = images().findIndex(
          (image) => image.id === imageId,
        );

        if (lastImageIdx === undefined || currentImageIdx === undefined) {
          return;
        }

        let startIdx = Math.min(lastImageIdx, currentImageIdx);
        let endIdx = Math.max(lastImageIdx, currentImageIdx);

        let range = images()
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
      if (tagMode()) {
        setSelectedImages([]);
      }
    },
    '$mod+m': (event) => {
      event.preventDefault();
      setEmptyTagMode((prev) => !prev);
    },
    '$mod+k': (event) => {
      if (!openImage()) {
        event.preventDefault();
        setSearchMode((prev) => !prev);
        searchInputRef.focus();
      }
    },
    '$mod+u': validateEvent(() => navigate('/upload')),
    o: validateEvent(() => {
      if (openImage()) {
        setOriginalQuality((o) => !o);
        showQualityHint();
      }
    }),
  });

  onCleanup(cleanup);

  // Scroll to top of page on refresh
  window.onbeforeunload = function () {
    window.scrollTo(0, 0);
  };

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

  createEffect(() => {
    if (images().length === 0) {
      return;
    }

    let lazyloadImages = document.querySelectorAll('img');

    imagesObserver = new IntersectionObserver(
      (entries, _) => {
        entries.forEach((entry) => {
          if (!entry.isIntersecting || data.loading) {
            return;
          }
          var image = entry.target;
          image.setAttribute('src', image.getAttribute('data-src'));
          image.classList.remove('lazy');
          imagesObserver.unobserve(image);
        });
      },
      { rootMargin: '100%' },
    );

    lazyloadImages.forEach(function (image) {
      imagesObserver.observe(image);
    });
  });

  createEffect(() => {
    let loadingRef = document.querySelector('#loading-ref');

    loadingObserver = new IntersectionObserver(
      (entries, _) => {
        entries.forEach((entry) => {
          if (!entry.isIntersecting || data.loading || !data().has_more) {
            return;
          }
          setPage((page) => page + 1);
        });
      },
      { rootMargin: '100%' },
    );

    loadingObserver.observe(loadingRef);
  });

  onCleanup(() => {
    imagesObserver?.disconnect();
    loadingObserver?.disconnect();
  });

  return (
    <div>
      <Show when={openImage()}>
        <Lightbox
          imageId={openImage()}
          images={images()}
          close={closeLightbox}
          originalQuality={originalQuality()}
        />
      </Show>
      <Show when={qualityHint()}>
        <div class="fixed bottom-2 right-2 text-xs">
          <Show when={originalQuality()} fallback={'medium'}>
            original
          </Show>
        </div>
      </Show>
      <Show when={searchMode() || isMobile()}>
        <div class="w-full">
          <div class="flex flex-row bg-white border-b border-black rounded-sm w-full h-12">
            <div class="pr-2 border-r border-black w-60 p-2 pt-3">
              <p class="text-sm text-gray-700">
                matched images: {data()?.total}
              </p>
            </div>
            <div class="p-2 flex w-full flex-row items-start">
              <input
                ref={searchInputRef}
                value={searchTerm()}
                class="p-1.5 w-full mx-1 outline-none text-xs"
                placeholder="search"
                autofocus
                onInput={(e) => {
                  searchDebounced(e.currentTarget.value);
                }}
              />
            </div>
          </div>
        </div>
      </Show>
      <Show when={images().length === 0}>
        <div class="flex w-full h-96">
          <div class="m-auto flex flex-col gap-4">
            <h1 class="text-4xl text-center ">No images found</h1>
            <p class="text-center">
              Press <span class="font-bold">cmd+u</span> to upload
            </p>
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
      <div class="flex flex-col w-full bg-white">
        <div class="w-full grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 p-2 md:p-4 gap-x-2 gap-y-4 max-w-screen-2xl mx-auto">
          <For each={images()}>
            {(image) => (
              <div
                class={`w-full flex items-center ${
                  selectedImages().includes(image?.id) && tagMode()
                    ? 'outline outline-3 outline-offset-2 outline-blue-600'
                    : ''
                }`}
              >
                <img
                  class="lazy object-cover aspect-[4/3] max-h-screen mx-auto my-auto"
                  id={image?.id}
                  onClick={(e) => onClickImage(image?.id, e)}
                  data-src={`/api/images/${image?.id}?quality=small`}
                />
              </div>
            )}
          </For>
        </div>
      </div>
      <div id="loading-ref" class="p-4"></div>
    </div>
  );
};

export default View;
