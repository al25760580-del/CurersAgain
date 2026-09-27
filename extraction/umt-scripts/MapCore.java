import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.listing.*;
import ghidra.program.model.address.*;
import ghidra.program.model.symbol.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;
public class MapCore extends GhidraScript {
 public void run() throws Exception {
  String base=getScriptArgs()[0]; File out=new File(base,"exports-core");out.mkdirs();
  DecompInterface dec=new DecompInterface();dec.openProgram(currentProgram);
  Set<Address> done=new HashSet<>();int count=0;
  try(PrintWriter index=new PrintWriter(new File(out,"references.tsv"))) {
   index.println("string_name\tstring_address\treference\tfunction\tentry");
   for(String line:Files.readAllLines(Path.of(base,"core-targets.tsv"))) {
    String[] v=line.split("\t");Address a=toAddr(v[0]);ReferenceIterator it=currentProgram.getReferenceManager().getReferencesTo(a);
    while(it.hasNext()) {
     Reference ref=it.next();Function f=getFunctionContaining(ref.getFromAddress());
     index.println(v[1]+"\t"+a+"\t"+ref.getFromAddress()+"\t"+(f==null?"DATA":f.getName())+"\t"+(f==null?"":f.getEntryPoint()));
     if(f!=null && done.add(f.getEntryPoint())) {
      DecompileResults r=dec.decompileFunction(f,40,monitor);
      if(r.decompileCompleted()) {try(PrintWriter p=new PrintWriter(new File(out,f.getEntryPoint()+".c"))) {p.println("/* Candidate referencing "+v[1]+"; identity not yet confirmed. */");p.println(r.getDecompiledFunction().getC());}count++;}
     }
    }
   }
  }finally{dec.dispose();}
  println("CORE MAP COMPLETE: decompiled="+count);
 }
}
