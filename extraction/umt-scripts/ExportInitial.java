import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.listing.*;
import java.io.*;
public class ExportInitial extends GhidraScript {
 public void run() throws Exception {
  File dir=new File(getScriptArgs()[0]); dir.mkdirs();
  int total=0, exported=0;
  DecompInterface dec=new DecompInterface(); dec.openProgram(currentProgram);
  try(PrintWriter index=new PrintWriter(new File(dir,"functions.tsv"));PrintWriter code=new PrintWriter(new File(dir,"initial-pseudocode.c"))) {
   index.println("address\tname\tsize");
   code.println("/* Initial native pseudocode, not original source. Bounded sample of up to 100 functions. */");
   FunctionIterator it=currentProgram.getFunctionManager().getFunctions(true);
   while(it.hasNext() && !monitor.isCancelled()) {
    Function f=it.next();total++;
    index.println(f.getEntryPoint()+"\t"+f.getName()+"\t"+f.getBody().getNumAddresses());
    if(exported<100 && !f.isExternal() && !f.isThunk() && f.getBody().getNumAddresses()<20000) {
     DecompileResults r=dec.decompileFunction(f,15,monitor);
     if(r.decompileCompleted() && r.getDecompiledFunction()!=null) {
      code.println("\n/* "+f.getEntryPoint()+" */\n"+r.getDecompiledFunction().getC());exported++;
     }
    }
   }
  } finally {dec.dispose();}
  println("EXPORT COMPLETE: indexed="+total+" decompiled="+exported);
 }
}
