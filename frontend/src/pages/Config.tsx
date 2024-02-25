import { createSignal, type Component } from 'solid-js';

const Config: Component = () => {
  const [bucket, setBucket] = createSignal('');
  const [endpoint, setEndpoint] = createSignal('');
  const [region, setRegion] = createSignal('');
  const [accessKey, setAccessKey] = createSignal('');
  const [secretKey, setSecretKey] = createSignal('');
  const [, setError] = createSignal('');

  const submit = (event: Event): void => {
    event.preventDefault();
    fetch('/user/config', {
      body: JSON.stringify({
        bucket: bucket(),
        endpoint: endpoint(),
        region: region(),
        access_key: accessKey(),
        secret_key: secretKey(),
      }),
      method: 'PATCH',
      headers: {
        'content-type': 'application/json',
      },
    })
      .then((response) => {
        if (!response.ok) {
          setError(response.statusText);
        }
      })
      .catch((error: Error) => setError(error.message));
  };

  return (
    <div class="mx-auto flex flex-col w-1/4 min-w-96 pt-12">
      <div class="flex flex-row gap-4 items-baseline">
        <p class="text-4xl md:text-6xl mb-4 text-black dark:text-white font-extrabold">
          Config
        </p>
      </div>
      <form onSubmit={submit} class="w-full flex flex-col gap-6 mt-12">
        <label class="block">
          <span class="text-sm text-gray-700">Bucket</span>
          <input
            type="text"
            class="focus:outline-none mt-0 block w-full border-0 border-b-2 border-dotted border-gray-400 px-0.5 focus:border-black focus:ring-0"
            placeholder=""
            value={bucket()}
            onChange={(event) => setBucket(event?.currentTarget?.value)}
          />
        </label>
        <label class="block">
          <span class="text-sm text-gray-700">Region</span>
          <input
            type="text"
            class="focus:outline-none mt-0 block w-full border-0 border-b-2 border-dotted border-gray-400 px-0.5 focus:border-black focus:ring-0"
            placeholder=""
            value={region()}
            onChange={(event) => setRegion(event?.currentTarget?.value)}
          />
        </label>
        <label class="block">
          <span class="text-sm text-gray-700">Endpoint</span>
          <input
            type="text"
            class="focus:outline-none mt-0 block w-full border-0 border-b-2 border-dotted border-gray-400 px-0.5 focus:border-black focus:ring-0"
            placeholder=""
            value={endpoint()}
            onChange={(event) => setEndpoint(event?.currentTarget?.value)}
          />
        </label>
        <label class="block">
          <span class="text-sm text-gray-700">Access Key</span>
          <input
            type="password"
            class="focus:outline-none mt-0 block w-full border-0 border-b-2 border-dotted border-gray-400 px-0.5 focus:border-black focus:ring-0"
            value={accessKey()}
            onChange={(event) => setAccessKey(event?.currentTarget?.value)}
          />
        </label>
        <label class="block">
          <span class="text-sm text-gray-700">Secret Key</span>
          <input
            type="password"
            class="focus:outline-none mt-0 block w-full border-0 border-b-2 border-dotted border-gray-400 px-0.5 focus:border-black focus:ring-0"
            value={secretKey()}
            onChange={(event) => setSecretKey(event?.currentTarget?.value)}
          />
        </label>
        <button
          type="submit"
          class="mt-8 rounded-sm bg-white border border-black py-2
                    uppercase text-black hover:shadow-[6px_6px_0_#00000020] 
                    transition-all duration-75"
        >
          <div class="relative">
            <span>Save</span>
          </div>
        </button>
      </form>
    </div>
  );
};

export default Config;
