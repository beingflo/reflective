import { useNavigate } from '@solidjs/router';
import { createSignal, Show, type Component } from 'solid-js';

const Signup: Component = () => {
  const [username, setUsername] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [error, setError] = createSignal('');
  const [success, setSuccess] = createSignal(false);
  const [loading, setLoading] = createSignal(false);

  const submit = (event: Event): void => {
    event.preventDefault();
    setError('');
    setSuccess(false);

    if (!username() || !password()) {
      setError('Please enter a username and password.');
      return;
    }

    setLoading(true);

    fetch('/api/auth/signup', {
      body: JSON.stringify({ username: username(), password: password() }),
      method: 'POST',
      headers: {
        'content-type': 'application/json',
      },
    })
      .then((response) => {
        if (response.ok) {
          setSuccess(true);
          setError('');
        } else {
          setError(response.statusText);
          setSuccess(false);
        }
      })
      .catch((error: Error) => {
        setError(error.message);
      })
      .finally(() => {
        setLoading(false);
      });
  };

  return (
    <div class="mx-auto flex flex-col w-1/4 min-w-96 pt-8 px-4 md:pt-12 md:px-0">
      <div class="flex flex-row gap-4 items-baseline">
        <p class="text-4xl md:text-6xl mb-4 text-black dark:text-white font-extrabold">
          Signup
        </p>
        <a
          href="/login"
          class="text-md md:text-lg text-gray-800 dark:text-white w-fit h-fit"
        >
          Login
        </a>
      </div>
      <form onSubmit={submit} class="w-full flex flex-col gap-6 mt-12">
        <Show when={error()}>
          <div class="text-rose-700">Error: {error()}</div>
        </Show>
        <Show when={success()}>
          <div class="text-emerald-700">
            Account creation successful. Please{' '}
            <a href="/login" class="text-blue-600 underline">
              login
            </a>
            .
          </div>
        </Show>
        <label class="block">
          <span class="text-sm text-gray-700">Username</span>
          <input
            type="text"
            class="focus:outline-none mt-1 block w-full border border-black p-1 px-2 focus:border-black focus:ring-0 placeholder:text-sm"
            placeholder="Enter your username"
            value={username()}
            onChange={(event) => setUsername(event?.currentTarget?.value)}
          />
        </label>
        <label class="block">
          <span class="text-sm text-gray-700">Password</span>
          <input
            type="password"
            class="focus:outline-none mt-1 block w-full border border-black p-1 px-2 focus:border-black focus:ring-0 placeholder:text-sm"
            placeholder="Enter your password"
            value={password()}
            onChange={(event) => setPassword(event?.currentTarget?.value)}
          />
        </label>
        <button
          type="submit"
          class="mt-8 rounded-sm bg-white border border-black py-2
                    uppercase text-black hover:shadow-[6px_6px_0_#00000020] 
                    transition-all duration-75"
          disabled={loading()}
        >
          <div class="relative">
            <Show when={loading()} fallback={<span>Signup</span>}>
              <span class="flex gap-x-1 justify-center">
                <span class="animate-bounce">.</span>
                <span class="animate-bounce delay-200">.</span>
                <span class="animate-bounce delay-400">.</span>
              </span>
            </Show>
          </div>
        </button>
      </form>
    </div>
  );
};

export default Signup;
