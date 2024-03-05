import { Route, Router } from '@solidjs/router';
import type { Component } from 'solid-js';
import Login from './pages/Login';
import Config from './pages/Config';
import View from './pages/View';
import Upload from './pages/Upload';
import Signup from './pages/Signup';
import NotFound from './pages/NotFound';
import { StoreProvider } from './store';

const App: Component = () => {
  return (
    <StoreProvider>
      <Router>
        <Route path="/" component={View} />
        <Route path="/login" component={Login} />
        <Route path="/signup" component={Signup} />
        <Route path="/config" component={Config} />
        <Route path="/upload" component={Upload} />
        <Route path="*404" component={NotFound} />
      </Router>
    </StoreProvider>
  );
};

export default App;
