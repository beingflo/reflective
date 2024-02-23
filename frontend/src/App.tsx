import { Route, Router } from '@solidjs/router';
import type { Component } from 'solid-js';
import Login from './pages/Login';
import Config from './pages/Config';
import View from './pages/View';
import Upload from './pages/Upload';

const App: Component = () => {
  return (
    <Router>
      <Route path="/" component={View} />
      <Route path="/login" component={Login} />
      <Route path="/config" component={Config} />
      <Route path="/upload" component={Upload} />
    </Router>
  );
};

export default App;
