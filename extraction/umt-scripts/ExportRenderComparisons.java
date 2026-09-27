import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.listing.*;
import ghidra.program.model.pcode.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;
public class ExportRenderComparisons extends GhidraScript {
 public void run() throws Exception {
 String base=getScriptArgs()[0];File out=new File(base,"render-investigation/decomp");out.mkdirs();
 for(String mode:new String[]{"decompile","normalize","firstpass"}){
 DecompInterface d=new DecompInterface();d.setSimplificationStyle(mode);d.toggleCCode(mode.equals("decompile"));d.toggleSyntaxTree(true);d.openProgram(currentProgram);
 File dir=new File(out,mode);dir.mkdirs();
 for(String line:Files.readAllLines(Path.of(base,"render-investigation/targets.tsv"))){
 String[] v=line.split("\t");Function f=getFunctionAt(toAddr(v[0]));if(f==null){println("MISSING "+line);continue;}
 DecompileResults r=d.decompileFunction(f,40,monitor);String name=v[1];
 if(r.decompileCompleted()){
 if(r.getDecompiledFunction()!=null)try(PrintWriter w=new PrintWriter(new File(dir,name+".c"))){w.println("/* mode="+mode+" entry="+f.getEntryPoint()+"; same program, not an independent decompiler */");w.println(r.getDecompiledFunction().getC());}
 if(r.getHighFunction()!=null)try(PrintWriter w=new PrintWriter(new File(dir,name+".pcode.tsv"))){Iterator<PcodeOpAST> it=r.getHighFunction().getPcodeOps();while(it.hasNext()){PcodeOpAST op=it.next();w.println(op.getSeqnum().getTarget()+"\t"+op);}}
 println("EXPORTED "+mode+" "+name);
 }else println("FAILED "+mode+" "+name+" "+r.getErrorMessage());
 if(mode.equals("decompile"))try(PrintWriter w=new PrintWriter(new File(out,name+".asm.tsv"))){InstructionIterator it=currentProgram.getListing().getInstructions(f.getBody(),true);while(it.hasNext()){Instruction i=it.next();w.println(i.getAddress()+"\t"+i);for(PcodeOp op:i.getPcode())w.println(i.getAddress()+"\tRAW_PCODE\t"+op);}}
 }d.dispose();}
 println("COMPARISON_EXPORT_COMPLETE");
 }
}
