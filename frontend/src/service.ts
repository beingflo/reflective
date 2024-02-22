export const login = async (username: string, password: string) => {
  const response = await fetch('/auth/login', {
    body: JSON.stringify({ username, password }),
  });
  // TODO handle response
};

export const signup = async (username: string, password: string) => {
  const response = await fetch('/auth/signup', {
    body: JSON.stringify({ username, password }),
  });
  // TODO handle response
};
