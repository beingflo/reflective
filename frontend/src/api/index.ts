import { UpdateConfigRequest } from '../types';

export const login = async (username: string, password: string) => {
  const response = await fetch('/auth/login', {
    body: JSON.stringify({ username, password }),
    method: 'POST',
  });
  // TODO handle response
};

export const signup = async (username: string, password: string) => {
  const response = await fetch('/auth/signup', {
    body: JSON.stringify({ username, password }),
    method: 'POST',
  });
  // TODO handle response
};

export const updateConfig = async (request: UpdateConfigRequest) => {
  const response = await fetch('/user/config', {
    body: JSON.stringify(request),
    method: 'PATCH',
  });
  // TODO handle response
};

export const getImageUploadLinks = async (number: number) => {
  const response = await fetch('/images/upload', {
    body: JSON.stringify({ number }),
    method: 'POST',
  });
  // TODO handle response
};

export const getImageList = async () => {
  const response = await fetch('/images');
  // TODO handle response
};
