import { QueryClient, QueryClientProvider } from 'react-query'
import './App.css'
import { Clients } from './Clients';
import { Domains } from './Domains';
import { css } from '@emotion/react';

const queryClient = new QueryClient();

function App() {
  return (
    <div css={css`width: 100%; height: 100%`}>
      <QueryClientProvider client={queryClient}>
        <AppGrid />
      </QueryClientProvider>
    </div>
  )
}

function AppGrid() {
  const gridStyle = css`
    display: grid;
    width: 100%;
    grid-template-columns: 50% 50%;
    grid-gap: 18px;
  `
  return (
    <div css={gridStyle}>
        <span><Clients /></span>
        <span><Domains /></span>
    </div>
  )
}


export default App
