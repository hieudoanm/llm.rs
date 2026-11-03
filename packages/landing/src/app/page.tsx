import Navbar from '../components/Navbar';
import Hero from '../components/Hero';
import Bands from '../components/Bands';
import WhyLLM from '../components/WhyLLM';
import Workflow from '../components/Workflow';
import BuiltDifferent from '../components/BuiltDifferent';
import Tools from '../components/Tools';
import GetStarted from '../components/GetStarted';
import Footer from '../components/Footer';

export default function Page() {
  return (
    <div className='relative overflow-hidden'>
      <div className='pointer-events-none absolute top-20 right-[2%] h-[520px] w-[520px] rounded-full bg-[radial-gradient(circle,#d9554420_0,transparent_68%)] blur-[1px]' />
      <div className='pointer-events-none absolute top-[600px] left-[-250px] h-[550px] w-[550px] rounded-full bg-[radial-gradient(circle,#28405d30_0,transparent_70%)] blur-[1px]' />
      <Navbar />
      <main>
        <Hero />
        <Bands />
        <WhyLLM />
        <Workflow />
        <BuiltDifferent />
        <Tools />
        <GetStarted />
      </main>
      <Footer />
    </div>
  );
}
