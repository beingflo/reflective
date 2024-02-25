import { useNavigate } from '@solidjs/router';
import { createSignal, type Component } from 'solid-js';

const Login: Component = () => {
  const [username, setUsername] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [, setError] = createSignal('');
  const navigate = useNavigate();

  const submit = (event: Event): void => {
    event.preventDefault();
    fetch('/api/auth/login', {
      body: JSON.stringify({ username: username(), password: password() }),
      method: 'POST',
      headers: {
        'content-type': 'application/json',
      },
    })
      .then((response) => {
        if (response.ok) {
          navigate('/');
        } else {
          setError(response.statusText);
        }
      })
      .catch((error: Error) => setError(error.message));
  };

  return (
    <div class="mx-auto flex flex-col w-1/4 min-w-96 pt-12">
      <div class="flex flex-row gap-4 items-baseline">
        <p class="text-4xl md:text-6xl mb-4 text-black dark:text-white font-extrabold">
          Login
        </p>
        <a
          href="/signup"
          class="text-md md:text-lg text-gray-800 dark:text-white w-fit h-fit"
        >
          Signup
        </a>
      </div>
      <form onSubmit={submit} class="w-full flex flex-col gap-6 mt-12">
        <label class="block">
          <span class="text-sm text-gray-700">Username</span>
          <input
            type="text"
            autofocus
            class="focus:outline-none mt-0 block w-full border-0 border-b-2 border-dotted border-gray-400 px-0.5 focus:border-black focus:ring-0"
            placeholder=""
            value={username()}
            onChange={(event) => setUsername(event?.currentTarget?.value)}
          />
        </label>
        <label class="block">
          <span class="text-sm text-gray-700">Password</span>
          <input
            type="password"
            class="focus:outline-none mt-0 block w-full border-0 border-b-2 border-dotted border-gray-400 px-0.5 focus:border-black focus:ring-0"
            value={password()}
            onChange={(event) => setPassword(event?.currentTarget?.value)}
          />
        </label>
        <button
          type="submit"
          class="mt-8 rounded-sm bg-white border border-black py-2
                    uppercase text-black hover:shadow-[6px_6px_0_#00000020] 
                    transition-all duration-75"
        >
          <div class="relative">
            <span>Login</span>
          </div>
        </button>
      </form>
    </div>
  );
};

export default Login;
