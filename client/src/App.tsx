import { QueryClient, QueryClientProvider } from 'react-query'
import './App.css'
import { Clients } from './Clients';

const queryClient = new QueryClient();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Clients />
    </QueryClientProvider>
  )
}


export default App
