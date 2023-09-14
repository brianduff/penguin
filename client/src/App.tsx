import { QueryClient, QueryClientProvider } from 'react-query'
import './App.css'
import { Clients } from './Clients';

const queryClient = new QueryClient();

function App() {
  return (
    <div>
      <QueryClientProvider client={queryClient}>
        <Clients />
      </QueryClientProvider>
    </div>
  )
}


export default App
